import { invoke } from "@tauri-apps/api/core";

export interface ConnectionConfig {
  host: string;
  port: string;
  database: string;
  username: string;
  password: string;
}

export interface ConnectionResult {
  success: boolean;
  message: string;
}

export interface TableInfo {
  name: string;
  schema: string;
}

export interface TableData {
  columns: string[];
  rows: any[][];
  total_rows: number;
}

export interface ForeignKeyInfo {
  column_name: string;
  foreign_table_schema: string;
  foreign_table_name: string;
  foreign_column_name: string;
}

export interface IndexInfo {
  index_name: string;
  table_schema: string;
  table_name: string;
  columns: string[];
  is_unique: boolean;
  is_primary: boolean;
  index_type: string;
  size_bytes: number;
}

export interface RowData {
  columns: string[];
  values: any[];
}

export const db = {
  testConnection: async (
    config: ConnectionConfig,
  ): Promise<ConnectionResult> => {
    return await invoke<ConnectionResult>("test_connection", { config });
  },

  connect: async (config: ConnectionConfig): Promise<ConnectionResult> => {
    return await invoke<ConnectionResult>("connect_to_db", { config });
  },

  getTables: async (): Promise<TableInfo[]> => {
    return await invoke<TableInfo[]>("get_tables");
  },

  getTableData: async (
    tableName: string,
    schema: string,
    limit?: number,
    offset?: number,
  ): Promise<TableData> => {
    return await invoke<TableData>("get_table_data", {
      tableName,
      schema,
      limit,
      offset,
    });
  },

  getForeignKeys: async (
    tableName: string,
    schema: string,
  ): Promise<ForeignKeyInfo[]> => {
    return await invoke<ForeignKeyInfo[]>("get_foreign_keys", {
      tableName,
      schema,
    });
  },

  getIndexes: async (
    tableName: string,
    schema: string,
  ): Promise<IndexInfo[]> => {
    return await invoke<IndexInfo[]>("get_indexes", {
      tableName,
      schema,
    });
  },

  getRowByPk: async (
    tableName: string,
    schema: string,
    pkColumn: string,
    pkValue: any,
  ): Promise<RowData> => {
    return await invoke<RowData>("get_row_by_pk", {
      tableName,
      schema,
      pkColumn,
      pkValue,
    });
  },

  disconnect: async (): Promise<void> => {
    return await invoke<void>("disconnect_db");
  },
};
