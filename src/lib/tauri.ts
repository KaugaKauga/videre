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

export const db = {
  testConnection: async (
    config: ConnectionConfig
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
    offset?: number
  ): Promise<TableData> => {
    return await invoke<TableData>("get_table_data", {
      tableName,
      schema,
      limit,
      offset,
    });
  },

  disconnect: async (): Promise<void> => {
    return await invoke<void>("disconnect_db");
  },
};
