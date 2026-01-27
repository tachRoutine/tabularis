export interface Pagination {
  page: number;
  page_size: number;
  total_rows: number;
}

export interface QueryResult {
  columns: string[];
  rows: unknown[][];
  affected_rows: number;
  truncated?: boolean;
  pagination?: Pagination;
}

export interface Tab {
  id: string;
  title: string;
  type: 'console' | 'table';
  query: string;
  result: QueryResult | null;
  error: string;
  executionTime: number | null;
  page: number;
  activeTable: string | null;
  pkColumn: string | null;
  isLoading?: boolean;
  connectionId: string;
}
