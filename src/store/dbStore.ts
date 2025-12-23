import { create } from "zustand";
import { db, TableInfo } from "@/lib/tauri";

interface DbStore {
  isConnected: boolean;
  tables: TableInfo[];
  isLoading: boolean;
  error: string | null;

  setConnected: (connected: boolean) => void;
  setTables: (tables: TableInfo[]) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;

  fetchTables: () => Promise<void>;
  disconnect: () => Promise<void>;
}

export const useDbStore = create<DbStore>((set) => ({
  isConnected: false,
  tables: [],
  isLoading: false,
  error: null,

  setConnected: (connected) => set({ isConnected: connected }),

  setTables: (tables) => set({ tables }),

  setLoading: (loading) => set({ isLoading: loading }),

  setError: (error) => set({ error }),

  fetchTables: async () => {
    set({ isLoading: true, error: null });
    try {
      const tables = await db.getTables();
      set({ tables, isConnected: true, isLoading: false });
    } catch (error) {
      set({
        error: `Failed to fetch tables: ${error}`,
        isLoading: false,
        isConnected: false
      });
    }
  },

  disconnect: async () => {
    try {
      await db.disconnect();
      set({
        isConnected: false,
        tables: [],
        error: null
      });
    } catch (error) {
      set({ error: `Failed to disconnect: ${error}` });
    }
  },
}));
