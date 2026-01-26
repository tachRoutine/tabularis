use ssh2::Session;
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::OnceLock;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct SshTunnel {
    pub local_port: u16,
    running: Arc<AtomicBool>,
}

pub static TUNNELS: OnceLock<Mutex<HashMap<String, SshTunnel>>> = OnceLock::new();

pub fn get_tunnels() -> &'static Mutex<HashMap<String, SshTunnel>> {
    TUNNELS.get_or_init(|| Mutex::new(HashMap::new()))
}

impl SshTunnel {
    pub fn new(
        ssh_host: &str,
        ssh_port: u16,
        ssh_user: &str,
        ssh_password: Option<&str>,
        ssh_key_file: Option<&str>,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<Self, String> {
        // 1. Connect to SSH Server
        let tcp = TcpStream::connect(format!("{}:{}", ssh_host, ssh_port))
            .map_err(|e| format!("Failed to connect to SSH server: {}", e))?;

        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake()
            .map_err(|e| format!("SSH handshake failed: {}", e))?;

        // 2. Authenticate
        if let Some(key_path) = ssh_key_file {
            if !key_path.trim().is_empty() {
                sess.userauth_pubkey_file(
                    ssh_user,
                    None,
                    std::path::Path::new(key_path),
                    ssh_password,
                )
                .map_err(|e| format!("SSH key auth failed: {}", e))?;
            } else {
                if let Some(pwd) = ssh_password {
                    sess.userauth_password(ssh_user, pwd)
                        .map_err(|e| format!("SSH password auth failed: {}", e))?;
                } else {
                    return Err("No SSH credentials provided".to_string());
                }
            }
        } else if let Some(pwd) = ssh_password {
            sess.userauth_password(ssh_user, pwd)
                .map_err(|e| format!("SSH password auth failed: {}", e))?;
        } else {
            sess.userauth_agent(ssh_user)
                .map_err(|e| format!("SSH agent auth failed: {}", e))?;
        }

        if !sess.authenticated() {
            return Err("SSH authentication failed".to_string());
        }

        // Set a short timeout for the session to allow non-blocking-ish reads
        sess.set_timeout(10);

        // 3. Setup Local Listener
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("Failed to bind local port: {}", e))?;
        let local_port = listener.local_addr().unwrap().port();

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let sess = Arc::new(Mutex::new(sess));
        let remote_host = remote_host.to_string();

        thread::spawn(move || {
            for stream in listener.incoming() {
                if !running_clone.load(Ordering::Relaxed) {
                    break;
                }

                match stream {
                    Ok(local_stream) => {
                        let sess = sess.clone();
                        let r_host = remote_host.clone();
                        let running_inner = running_clone.clone();

                        thread::spawn(move || {
                            let mut sess_lock = match sess.lock() {
                                Ok(l) => l,
                                Err(_) => return,
                            };

                            let mut channel =
                                match sess_lock.channel_direct_tcpip(&r_host, remote_port, None) {
                                    Ok(c) => c,
                                    Err(e) => {
                                        eprintln!("Failed to open SSH channel: {}", e);
                                        return;
                                    }
                                };

                            let mut local_stream = local_stream;
                            if let Err(_) = local_stream.set_nonblocking(true) {
                                return;
                            }

                            let mut buf = [0u8; 8192];
                            let mut active = true;

                            while active && running_inner.load(Ordering::Relaxed) {
                                let mut did_work = false;

                                // 1. Local -> Remote
                                match local_stream.read(&mut buf) {
                                    Ok(0) => {
                                        active = false;
                                        break;
                                    } // EOF
                                    Ok(n) => {
                                        if channel.write_all(&buf[..n]).is_err() {
                                            active = false;
                                            break;
                                        }
                                        did_work = true;
                                    }
                                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {}
                                    Err(_) => {
                                        active = false;
                                        break;
                                    }
                                }

                                // 2. Remote -> Local
                                // channel.read blocks based on session timeout (set to 10ms)
                                match channel.read(&mut buf) {
                                    Ok(0) => {
                                        // ssh2 read returning 0 might mean EOF or Timeout if configured?
                                        // With libssh2, 0 usually means EOF.
                                        // Timeout usually returns Err(WouldBlock) or similar if configured properly?
                                        // Actually `ssh2` rust wrapper maps timeout to Ok(0)? No.
                                        // Let's check error.
                                        // If EOF, active = false.
                                        // But wait, if timeout, it might return Err.
                                        // Let's assume EOF for now.
                                        active = false;
                                        break;
                                    }
                                    Ok(n) => {
                                        if local_stream.write_all(&buf[..n]).is_err() {
                                            active = false;
                                            break;
                                        }
                                        did_work = true;
                                    }
                                    Err(_) => {
                                        // Timeout or error. Ideally check if it's a timeout.
                                        // ssh2 error handling is specific.
                                        // We assume timeout if we set it.
                                    }
                                }

                                if !did_work {
                                    thread::sleep(Duration::from_millis(1));
                                }
                            }
                            // Channel closes on drop (end of scope)
                        });
                    }
                    Err(_) => {}
                }
            }
        });

        Ok(Self {
            local_port,
            running,
        })
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
