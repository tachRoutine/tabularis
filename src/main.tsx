import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';
import { DatabaseProvider } from './contexts/DatabaseProvider';
import { SettingsProvider } from './contexts/SettingsProvider';
import { SavedQueriesProvider } from './contexts/SavedQueriesProvider';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <SettingsProvider>
      <DatabaseProvider>
        <SavedQueriesProvider>
          <App />
        </SavedQueriesProvider>
      </DatabaseProvider>
    </SettingsProvider>
  </React.StrictMode>,
);
