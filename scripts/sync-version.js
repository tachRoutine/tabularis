import { readFileSync, writeFileSync } from 'fs';
import { resolve } from 'path';

// Percorsi dei file
const paths = {
  package: resolve('package.json'),
  tauri: resolve('src-tauri/tauri.conf.json'),
  cargo: resolve('src-tauri/Cargo.toml')
};

// 1. Leggi la nuova versione da package.json (che è già stato aggiornato da npm version)
const pkg = JSON.parse(readFileSync(paths.package, 'utf-8'));
const newVersion = pkg.version;

console.log(`🔄 Syncing version to ${newVersion}...`);

// 2. Aggiorna tauri.conf.json
const tauriConf = JSON.parse(readFileSync(paths.tauri, 'utf-8'));
tauriConf.version = newVersion;
// Aggiorna anche la versione nel nodo package se presente (Tauri v2 non lo usa sempre, ma per sicurezza)
if (tauriConf.package) tauriConf.package.version = newVersion;
writeFileSync(paths.tauri, JSON.stringify(tauriConf, null, 2));
console.log('✅ Updated tauri.conf.json');

// 3. Aggiorna Cargo.toml
let cargo = readFileSync(paths.cargo, 'utf-8');
// Usa una regex per sostituire solo la versione nel blocco [package]
// Cerca "version =" seguito da virgolette, ma solo nelle prime righe per evitare di cambiare le dipendenze
cargo = cargo.replace(
  /^version = ".*"/m,
  `version = "${newVersion}"`
);
writeFileSync(paths.cargo, cargo);
console.log('✅ Updated Cargo.toml');
