import { create } from "zustand";
import { load, Store } from "@tauri-apps/plugin-store";

export interface SavedConnection {
  id: string;
  name: string;
  host: string;
  port: string;
  database: string;
  username: string;
  savedAt: number;
}

interface ConnectionStore {
  connections: SavedConnection[];
  isLoaded: boolean;

  saveConnection: (
    connection: Omit<SavedConnection, "id" | "savedAt">,
  ) => Promise<void>;
  removeConnection: (id: string) => Promise<void>;
}

// Singleton store instance - initialized lazily
let storeInstance: Store | null = null;
let initPromise: Promise<Store> | null = null;

async function getStore(): Promise<Store> {
  if (storeInstance) return storeInstance;
  if (initPromise) return initPromise;

  initPromise = load("connections.json").then((store) => {
    storeInstance = store;
    return store;
  });

  return initPromise;
}

export const useConnectionStore = create<ConnectionStore>((set, get) => {
  // Auto-initialize on store creation
  getStore()
    .then(async (store) => {
      const savedConnections =
        await store.get<SavedConnection[]>("connections");
      set({ connections: savedConnections || [], isLoaded: true });
    })
    .catch((error) => {
      console.error("Failed to initialize connection store:", error);
      set({ isLoaded: true });
    });

  return {
    connections: [],
    isLoaded: false,

    saveConnection: async (connection) => {
      const store = await getStore();
      const { connections } = get();

      // Check if connection already exists
      const existingIndex = connections.findIndex(
        (c) =>
          c.host === connection.host &&
          c.port === connection.port &&
          c.database === connection.database &&
          c.username === connection.username,
      );

      let updatedConnections: SavedConnection[];

      if (existingIndex !== -1) {
        // Update existing - move to top with new timestamp
        updatedConnections = [...connections];
        updatedConnections.splice(existingIndex, 1);
        updatedConnections.unshift({
          ...connections[existingIndex],
          name: connection.name,
          savedAt: Date.now(),
        });
      } else {
        // Add new connection at top
        updatedConnections = [
          {
            ...connection,
            id: crypto.randomUUID(),
            savedAt: Date.now(),
          },
          ...connections,
        ];
      }

      // Keep only the last 10 connections
      updatedConnections = updatedConnections.slice(0, 10);

      await store.set("connections", updatedConnections);
      await store.save();
      set({ connections: updatedConnections });
    },

    removeConnection: async (id) => {
      const store = await getStore();
      const { connections } = get();

      const updatedConnections = connections.filter((c) => c.id !== id);
      await store.set("connections", updatedConnections);
      await store.save();
      set({ connections: updatedConnections });
    },
  };
});
