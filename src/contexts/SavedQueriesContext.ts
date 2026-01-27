import { createContext } from 'react';

export interface SavedQuery {
  id: string;
  name: string;
  sql: string;
  connection_id: string;
}

export interface SavedQueriesContextType {
  queries: SavedQuery[];
  isLoading: boolean;
  saveQuery: (name: string, sql: string) => Promise<void>;
  updateQuery: (id: string, name: string, sql: string) => Promise<void>;
  deleteQuery: (id: string) => Promise<void>;
  refreshQueries: () => Promise<void>;
}

export const SavedQueriesContext = createContext<SavedQueriesContextType | undefined>(undefined);
