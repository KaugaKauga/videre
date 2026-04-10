import { create } from "zustand";
import {
  db,
  TableInfo,
  ForeignKeyInfo,
  IndexInfo,
  RoleInfo,
  TablePrivilege,
} from "@/lib/tauri";

// Map key is "schema.table" for efficient lookup
type ForeignKeyMap = Record<string, ForeignKeyInfo[]>;
type IndexMap = Record<string, IndexInfo[]>;

interface DbStore {
  isConnected: boolean;
  tables: TableInfo[];
  foreignKeys: ForeignKeyMap;
  indexes: IndexMap;
  roles: RoleInfo[];
  tablePrivileges: TablePrivilege[];
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
  getIndexesForTable: (tableName: string, schema: string) => IndexInfo[];
  getPrivilegesForRole: (roleName: string) => TablePrivilege[];
  disconnect: () => Promise<void>;
}

export const useDbStore = create<DbStore>((set, get) => ({
  isConnected: false,
  tables: [],
  foreignKeys: {},
  indexes: {},
  roles: [],
  tablePrivileges: [],
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

      // Fetch foreign keys, indexes, and roles in parallel
      console.log("Fetching foreign keys, indexes, and roles...");
      const [fkResults, indexResults, roles, tablePrivileges] =
        await Promise.all([
          Promise.all(
            tables.map(async (table) => {
              const fks = await db.getForeignKeys(table.name, table.schema);
              return { key: `${table.schema}.${table.name}`, fks };
            }),
          ),
          Promise.all(
            tables.map(async (table) => {
              const idxs = await db.getIndexes(table.name, table.schema);
              return { key: `${table.schema}.${table.name}`, idxs };
            }),
          ),
          db.getRoles(),
          db.getTablePrivileges(),
        ]);

      // Build the FK map
      const foreignKeys: ForeignKeyMap = {};
      for (const { key, fks } of fkResults) {
        foreignKeys[key] = fks;
      }

      // Build the index map
      const indexes: IndexMap = {};
      for (const { key, idxs } of indexResults) {
        indexes[key] = idxs;
      }

      const totalFks = Object.values(foreignKeys).reduce(
        (sum, fks) => sum + fks.length,
        0,
      );
      const totalIndexes = Object.values(indexes).reduce(
        (sum, idxs) => sum + idxs.length,
        0,
      );
      console.log(`Loaded ${totalFks} foreign keys across all tables`);
      console.log(`Loaded ${totalIndexes} indexes across all tables`);
      console.log(`Loaded ${roles.length} roles`);
      console.log(`Loaded ${tablePrivileges.length} table privileges`);

      set({
        tables,
        foreignKeys,
        indexes,
        roles,
        tablePrivileges,
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

  getIndexesForTable: (tableName: string, schema: string) => {
    const key = `${schema}.${tableName}`;
    return get().indexes[key] || [];
  },

  getPrivilegesForRole: (roleName: string) => {
    return get().tablePrivileges.filter((p) => p.grantee === roleName);
  },

  disconnect: async () => {
    try {
      await db.disconnect();
      set({
        isConnected: false,
        tables: [],
        foreignKeys: {},
        indexes: {},
        roles: [],
        tablePrivileges: [],
        error: null,
      });
    } catch (error) {
      set({ error: `Failed to disconnect: ${error}` });
    }
  },
}));
