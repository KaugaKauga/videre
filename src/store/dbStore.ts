import { create } from "zustand";
import { db, TableInfo, ForeignKeyInfo } from "@/lib/tauri";

// Map key is "schema.table" for efficient lookup
type ForeignKeyMap = Record<string, ForeignKeyInfo[]>;

interface DbStore {
  isConnected: boolean;
  tables: TableInfo[];
  foreignKeys: ForeignKeyMap;
  isLoading: boolean;
  error: string | null;

  setConnected: (connected: boolean) => void;
  setTables: (tables: TableInfo[]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  fetchDatabaseMetadata: () => Promise<void>;
  getForeignKeysForTable: (
    tableName: string,
    schema: string,
  ) => ForeignKeyInfo[];
  disconnect: () => Promise<void>;
}

export const useDbStore = create<DbStore>((set, get) => ({
  isConnected: false,
  tables: [],
  foreignKeys: {},
  isLoading: false,
  error: null,

  setConnected: (connected) => set({ isConnected: connected }),

  setTables: (tables) => set({ tables }),

  setLoading: (loading) => set({ isLoading: loading }),

  setError: (error) => set({ error }),

  fetchDatabaseMetadata: async () => {
    set({ isLoading: true, error: null });
    try {
      console.log("Fetching database metadata...");

      // Fetch all tables
      const tables = await db.getTables();
      console.log(
        `Found ${tables.length} tables:`,
        tables.map((t) => `${t.schema}.${t.name}`),
      );

      // Fetch foreign keys for all tables in parallel
      console.log("Fetching foreign keys for all tables...");
      const fkResults = await Promise.all(
        tables.map(async (table) => {
          const fks = await db.getForeignKeys(table.name, table.schema);
          return { key: `${table.schema}.${table.name}`, fks };
        }),
      );

      // Build the FK map
      const foreignKeys: ForeignKeyMap = {};
      for (const { key, fks } of fkResults) {
        foreignKeys[key] = fks;
      }

      const totalFks = Object.values(foreignKeys).reduce(
        (sum, fks) => sum + fks.length,
        0,
      );
      console.log(`Loaded ${totalFks} foreign keys across all tables`);

      set({
        tables,
        foreignKeys,
        isConnected: true,
        isLoading: false,
      });
    } catch (error) {
      console.error("Failed to fetch database metadata:", error);
      set({
        error: `Failed to fetch database metadata: ${error}`,
        isLoading: false,
        isConnected: false,
      });
    }
  },

  getForeignKeysForTable: (tableName: string, schema: string) => {
    const key = `${schema}.${tableName}`;
    return get().foreignKeys[key] || [];
  },

  disconnect: async () => {
    try {
      await db.disconnect();
      set({
        isConnected: false,
        tables: [],
        foreignKeys: {},
        error: null,
      });
    } catch (error) {
      set({ error: `Failed to disconnect: ${error}` });
    }
  },
}));
