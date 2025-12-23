# Daedalus API Documentation

This document describes all available Tauri commands that can be invoked from the frontend.

## Database Commands

### `test_connection`

Tests a database connection without storing it.

**Parameters:**
```typescript
{
  config: ConnectionConfig
}
```

**ConnectionConfig:**
```typescript
{
  host: string;      // Database host (e.g., "localhost")
  port: string;      // Database port (e.g., "5432")
  database: string;  // Database name
  username: string;  // Database username
  password: string;  // Database password
}
```

**Returns:**
```typescript
{
  success: boolean;  // Whether the connection succeeded
  message: string;   // Status message
}
```

**Example:**
```typescript
import { db } from "@/lib/tauri";

const result = await db.testConnection({
  host: "localhost",
  port: "5432",
  database: "mydb",
  username: "postgres",
  password: "password123"
});

if (result.success) {
  console.log("Connection successful!");
}
```

---

### `connect_to_db`

Establishes a database connection and stores it in the application state.

**Parameters:**
```typescript
{
  config: ConnectionConfig
}
```

**Returns:**
```typescript
{
  success: boolean;
  message: string;
}
```

**Example:**
```typescript
const result = await db.connect({
  host: "localhost",
  port: "5432",
  database: "mydb",
  username: "postgres",
  password: "password123"
});

if (result.success) {
  // Fetch tables or update UI
}
```

---

### `get_tables`

Fetches all tables from the connected database (excludes system schemas).

**Parameters:** None

**Returns:**
```typescript
TableInfo[]
```

**TableInfo:**
```typescript
{
  name: string;    // Table name
  schema: string;  // Schema name (e.g., "public")
}
```

**Example:**
```typescript
const tables = await db.getTables();
console.log(tables);
// [
//   { name: "users", schema: "public" },
//   { name: "posts", schema: "public" },
//   { name: "comments", schema: "blog" }
// ]
```

---

### `get_table_data`

Fetches paginated data from a specific table.

**Parameters:**
```typescript
{
  tableName: string;   // Name of the table
  schema: string;      // Schema name (e.g., "public")
  limit?: number;      // Number of rows to fetch (default: 100)
  offset?: number;     // Number of rows to skip (default: 0)
}
```

**Returns:**
```typescript
{
  columns: string[];        // Array of column names
  rows: any[][];           // 2D array of row data
  total_rows: number;      // Total count of rows in the table
}
```

**Example:**
```typescript
const data = await db.getTableData(
  "users",
  "public",
  50,   // limit
  0     // offset
);

console.log(data);
// {
//   columns: ["id", "name", "email", "created_at"],
//   rows: [
//     [1, "Alice", "alice@example.com", "2024-01-15"],
//     [2, "Bob", "bob@example.com", "2024-01-16"],
//     ...
//   ],
//   total_rows: 1523
// }
```

---

### `disconnect_db`

Disconnects from the current database and clears stored connection state.

**Parameters:** None

**Returns:** `void`

**Example:**
```typescript
await db.disconnect();
console.log("Disconnected from database");
```

---

## Data Types

### Supported PostgreSQL Types

The API automatically converts PostgreSQL types to JSON-compatible values:

- **String types** (`varchar`, `text`, etc.) → `string`
- **Integer types** (`int`, `bigint`, etc.) → `number`
- **Float types** (`real`, `double precision`) → `number`
- **Boolean** → `boolean`
- **NULL** → `null`
- **Other types** → Attempted as string, fallback to `null`

### NULL Handling

NULL values from the database are represented as `null` in the JSON response. The frontend can detect and display these appropriately (e.g., showing "NULL" in italic gray text).

---

## Error Handling

All commands return a `Result` type. Errors are returned as strings describing what went wrong.

**Common Errors:**

- `"Not connected to database"` - Attempting to fetch data without an active connection
- `"Connection failed: <reason>"` - Unable to connect to the database
- `"Failed to fetch tables: <reason>"` - Error querying information_schema
- `"Failed to fetch data: <reason>"` - Error querying table data

**Example Error Handling:**
```typescript
try {
  const tables = await db.getTables();
  setTables(tables);
} catch (error) {
  console.error("Error fetching tables:", error);
  setError(`Failed to fetch tables: ${error}`);
}
```

---

## State Management

The backend maintains a global state with:

- **Active Client** - The connected PostgreSQL client (if any)
- **Connection Config** - The current connection configuration

This state is managed using `Arc<Mutex<Option<T>>>` for thread-safe access across async operations.

---

## Security Considerations

1. **Local Storage** - All credentials and connections are stored only in application memory
2. **No Persistence** - Credentials are NOT saved to disk (you must reconnect on app restart)
3. **No Network Calls** - Connection details never leave your machine
4. **Direct Connection** - The app connects directly to the database (no proxy/server)

---

## Future Enhancements

Planned features for the API:

- [ ] Execute custom SQL queries
- [ ] Export data to CSV/JSON
- [ ] Table schema inspection (column types, constraints, etc.)
- [ ] Support for MySQL, SQLite, and other databases
- [ ] Connection pooling
- [ ] Saved connection profiles (encrypted)
- [ ] Query history
- [ ] Transaction support

---

## TypeScript Wrapper

The TypeScript wrapper (`src/lib/tauri.ts`) provides a clean API:

```typescript
export const db = {
  testConnection: async (config: ConnectionConfig): Promise<ConnectionResult>
  connect: async (config: ConnectionConfig): Promise<ConnectionResult>
  getTables: async (): Promise<TableInfo[]>
  getTableData: async (tableName: string, schema: string, limit?: number, offset?: number): Promise<TableData>
  disconnect: async (): Promise<void>
}
```

All commands use `invoke` from `@tauri-apps/api/core` under the hood.