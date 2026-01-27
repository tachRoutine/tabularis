import { createContext } from 'react';

export interface Settings {
  queryLimit: number;
}

export interface SettingsContextType {
  settings: Settings;
  updateSetting: <K extends keyof Settings>(key: K, value: Settings[K]) => void;
}

export const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

export const DEFAULT_SETTINGS: Settings = {
  queryLimit: 500,
};
