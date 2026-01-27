import { createContext } from 'react';

export interface TableInfo {
  name: string;
}

export interface SavedConnection {
  id: string;
  name: string;
  params: {
    driver: string;
    host?: string;
    database: string;
  };
}

export interface DatabaseContextType {
  activeConnectionId: string | null;
  activeDriver: string | null;
  activeTable: string | null;
  activeConnectionName: string | null;
  activeDatabaseName: string | null;
  tables: TableInfo[];
  isLoadingTables: boolean;
  connect: (connectionId: string) => Promise<void>;
  disconnect: () => void;
  setActiveTable: (table: string | null) => void;
  refreshTables: () => Promise<void>;
}

export const DatabaseContext = createContext<DatabaseContextType | undefined>(undefined);
