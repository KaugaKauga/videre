# Leptos Migration Plan

Migration of Videre's frontend from React/TypeScript to Leptos (Rust/WASM).

The Rust backend (`src-tauri/`) is **untouched** — only the frontend changes.

---

## Completed

### 1. Project Scaffolding
- [x] Backed up React frontend to `src-react/`
- [x] Backed up `index.html` → `index-react.html`, `tauri.conf.json` → `tauri.conf.react.json`
- [x] Created `src-leptos/` crate with `Cargo.toml` (leptos CSR + wasm-bindgen + serde)
- [x] Created `Trunk.toml` build config
- [x] Created new `index.html` for Trunk (links to WASM crate + `style.css`)
- [x] Updated `tauri.conf.json` to use Trunk (`beforeDevCommand`, `beforeBuildCommand`, `withGlobalTauri`)
- [x] Added `src-leptos/target` and `.trunk` to `.gitignore`

### 2. Tauri IPC Wrapper (`tauri.rs`)
- [x] Generic `invoke<T>()` — calls `window.__TAURI__.core.invoke` via wasm-bindgen
- [x] `invoke_void()` variant for commands that return nothing
- [x] Proper error handling (rejected promises → `Result::Err`)

### 3. Shared Types (`types.rs`)
- [x] All backend types mirrored: `ConnectionConfig`, `ConnectionResult`, `TableInfo`, `TableData`, `ForeignKeyInfo`, `IndexInfo`, `RoleInfo`, `TablePrivilege`, `RowData`
- [x] `Serialize` + `Deserialize` derives for serde-wasm-bindgen

### 4. Base Stylesheet (`style.css`)
- [x] CSS reset
- [x] Amethyst-haze theme tokens (light + dark) via CSS custom properties
- [x] Component classes: `.card`, `.btn`, `.btn-primary`, `.btn-secondary`, `.btn-ghost`, `.field`, `input`, `label`
- [x] Status messages: `.status-msg`, `.status-success`, `.status-error`
- [x] Connection page layout: `.connection-page`, `.connection-card`, `.connection-layout`
- [x] Recents sidebar: `.recents`, `.recent-item`, `.recent-delete`
- [x] Spinner animation

### 5. Connection Page (`connection.rs`)
- [x] Form with host / port / database / username / password (reactive signals)
- [x] Test Connection button with loading spinner
- [x] Connect button with loading spinner
- [x] Success / error status messages with icons
- [x] Calls backend via `tauri::invoke`

### 6. Connection Store (`connection_store.rs`)
- [x] `SavedConnection` struct with `#[serde(rename_all = "camelCase")]` (backward-compatible with React's stored JSON)
- [x] Tauri plugin-store IPC: `plugin:store|load`, `plugin:store|get`, `plugin:store|set`, `plugin:store|save`
- [x] Cached resource ID via `thread_local!` (only calls `load` once per session)
- [x] `ConnectionStore` with `RwSignal`s provided via Leptos context
- [x] `init()` — async load from disk on startup
- [x] `save_connection()` — dedup, bump to top, truncate to 10, persist
- [x] `remove_connection()` — remove by ID, persist
- [x] Recents sidebar in ConnectionPage — click to fill form, trash to delete
- [x] Added `store:allow-load` permission to Tauri capabilities

---

### 7. Database Store (equivalent of `dbStore.ts`)
The global reactive state that tracks connection status and all fetched metadata.

- [x] Create `db_store.rs`
- [x] Signals: `is_connected`, `tables`, `foreign_keys`, `indexes`, `roles`, `table_privileges`, `is_loading`, `error`
- [x] `fetch_database_metadata()` — calls `get_tables`, `get_foreign_keys`, `get_indexes`, `get_roles`, `get_table_privileges` sequentially (local IPC, no perf difference)
- [x] `disconnect()` — calls `disconnect_db`, resets all state
- [x] Lookup helpers: `get_foreign_keys_for_table()`, `get_indexes_for_table()`, `get_privileges_for_role()`
- [x] Provide via context alongside `ConnectionStore`
- [x] Wire into ConnectionPage — on successful connect, set connected + fetch metadata

### 8. App Shell — Sidebar
Port `features/shell/Sidebar.tsx`. A collapsible sidebar showing:

- [x] App header ("Videre" + database icon)
- [x] Tables list (from `DbStore.tables`) — click opens a table tab
- [x] Loading spinner when fetching metadata
- [x] "No tables found" empty state
- [x] Bottom section: Indexes, Roles links
- [x] Footer: Connection, Settings links
- [x] Style the sidebar using existing `--sidebar-*` CSS variables

### 9. App Shell — Tab Bar
Port `features/shell/TabBar.tsx`. A horizontal tab strip:

- [x] Tab types: `table`, `empty`, `settings`, `connection`, `indexes`, `roles`
- [x] Active tab indicator (bottom border)
- [x] Close button per tab (X icon, visible on hover)
- [x] Click to switch tabs

### 10. App Shell — Tab State & Routing
Port the tab management logic from `App.tsx`. No router — tabs are managed via signals.

- [x] `tabs: RwSignal<Vec<Tab>>` and `active_tab_id: RwSignal<Option<String>>`
- [x] `open_table_tab()` — reuse existing or replace empty tab
- [x] `open_singleton_tab()` — for settings, connection, indexes, roles (only one instance)
- [x] `open_empty_tab()` — "Untitled N" naming
- [x] `close_tab()` — activate last remaining tab on close
- [x] Content area renders the active tab's component (placeholders for table/settings/indexes/roles)
- [x] Full-screen ConnectionPage when not connected; Shell when connected

### 11. Keyboard Shortcuts
Port `hooks/useKeyboardShortcuts.ts`.

- [x] `Cmd/Ctrl + T` — new empty tab
- [x] `Cmd/Ctrl + W` — close active tab
- [x] `Cmd/Ctrl + 1-9` — switch to tab by index
- [x] Platform detection (Mac → Meta key, others → Ctrl)
- [x] `web_sys` keydown event listener via `wasm_bindgen::Closure` + `on_cleanup` for removal

### 18. Empty States
Port `features/empty/EmptyState.tsx` and `EmptyTab.tsx`.

- [x] `EmptyState` — "No table selected" with database icon (shown when no tabs)
- [x] `EmptyTab` — "Empty Tab" with keyboard shortcut hints (shown for blank tabs)

---

## Remaining

### 12. Table Page
Port `features/table/TablePage.tsx`. The core data browsing view.

- [x] Fetch table data on mount via `get_table_data` (with limit/offset)
- [x] Column headers with sort toggle (client-side sort via `DataTable` logic)
- [x] Pagination controls (prev/next, page indicator, total rows)
- [x] Foreign key display for the table (FK badge in headers, clickable FK cells)
- [x] FK detail side panel — slide-out panel fetches referenced row via `get_row_by_pk`
- [x] Loading / error states
- [x] Style: data table with sticky header, horizontal scroll

### 13. DataTable Component
Port `components/DataTable.tsx`. Generic sortable table.

- [x] Column definitions from `Vec<String>`, rows from `Vec<Vec<serde_json::Value>>`
- [x] Client-side sorting (asc → desc → none) with `compare_values` supporting null/number/string/bool
- [x] Sort indicator arrows in headers (▲/▼), inactive indicator on hover
- [x] "No results" empty state
- [x] Optional FK support via `fk_columns` + `fk_click` signal props
- [x] CSS for `.data-table` — sticky header, hover rows, proper cell padding

### 14. Indexes Page
Port `features/indexes/IndexesPage.tsx`.

- [x] Read indexes from `DbStore` (flattened from all per-table maps, sorted by schema/table/name)
- [x] Display in a table: index name, table, columns, unique (✓/—), primary (✓/—), type (badge), size (formatted bytes)
- [x] Empty state with list icon
- [x] Header with total index count across N tables

### 15. Roles Page
Port `features/roles/RolesPage.tsx`.

- [x] Read roles + privileges from `DbStore`
- [x] Split into Users (can_login) and Groups tables
- [x] Permission summaries per role (superuser / read-write / read-only / mixed / none) with colored badges
- [x] Users table: name (clickable), permissions, member of, conn limit, valid until
- [x] Groups table: name (clickable), permissions
- [x] Role detail side panel: properties, member-of badges, per-table privilege badges (SELECT/INSERT/UPDATE/DELETE)

### 16. Settings Page
Port `features/settings/SettingsPage.tsx`.

- [x] Theme selection: amethyst-haze, solar-dusk, nature (with color preview swatches)
- [x] Mode selection: light / dark (with sun/moon icons)
- [x] About section (version, app name)
- [x] Persists to localStorage
- [x] Applies theme by toggling CSS classes on `<html>`

### 17. Theme System
Port `lib/theme.ts`.

- [x] Create `theme.rs` with `apply_theme()`, `get_stored_theme()`, `set_stored_theme()`, etc.
- [x] Read/write localStorage via `web_sys::Storage`
- [x] Manipulate `document.documentElement.classList` via `web_sys`
- [x] Call `initialize_theme()` in `main.rs` before mount
- [x] Add solar-dusk and nature theme CSS variables to `style.css` (currently only amethyst-haze)

### 19. Cleanup
- [ ] Remove all `[store]` debug console.log statements from `connection_store.rs`
- [ ] Suppress unused type warnings with `#[allow(dead_code)]` or by actually using them
- [ ] Review and tighten `web-sys` feature flags
- [ ] Consider extracting SVG icons into a shared `icons.rs` module to reduce duplication

### 20. Final Validation
- [ ] Verify all Tauri IPC commands work: `test_connection`, `connect_to_db`, `get_tables`, `get_table_data`, `get_foreign_keys`, `get_indexes`, `get_roles`, `get_table_privileges`, `get_row_by_pk`, `disconnect_db`
- [ ] Verify plugin-store persistence survives app restart
- [ ] Test all three themes in both light and dark mode
- [ ] Test keyboard shortcuts
- [ ] Test pagination on large tables
- [ ] Test with no database connection (graceful empty states)
- [ ] Compare visual output with React version for parity

---

## File Map

```
src-leptos/
├── Cargo.toml
└── src/
    ├── main.rs               ✅  Entry point, mounts App, provides context
    ├── tauri.rs              ✅  IPC wrapper (invoke, invoke_void)
    ├── types.rs              ✅  Shared types mirroring backend
    ├── connection.rs         ✅  Connection page + recents sidebar
    ├── connection_store.rs   ✅  Persisted connection history (plugin-store)
    ├── db_store.rs           ✅  Global DB state (tables, indexes, roles, etc.)
    ├── tab_store.rs          ✅  Tab state management (open/close/switch)
    ├── shell.rs              ✅  App shell (sidebar + tabbar + content + keyboard shortcuts)
    ├── sidebar.rs            ✅  Sidebar component
    ├── tab_bar.rs            ✅  Tab bar component
    ├── empty.rs              ✅  EmptyState + EmptyTab components
    ├── table_page.rs         ✅  Table data browser with pagination + FK side panel
    ├── data_table.rs         ✅  Generic sortable table component with FK support
    ├── indexes_page.rs       ✅  Indexes viewer
    ├── roles_page.rs         ✅  Roles viewer with detail side panel
    ├── theme.rs              ✅  Theme/mode management (localStorage + classList)
    └── settings_page.rs      ✅  Settings (theme picker, about)

style.css                     ✅  Base styles + all three theme tokens (amethyst-haze, solar-dusk, nature)
index.html                    ✅  Trunk entry point
Trunk.toml                    ✅  Build config

src-react/                    📦  Backup of original React frontend (do not delete yet)
index-react.html              📦  Backup of original index.html
src-tauri/tauri.conf.react.json 📦  Backup of original Tauri config
```

## Rollback

To switch back to the React frontend:

```sh
cp src-tauri/tauri.conf.react.json src-tauri/tauri.conf.json
cp index-react.html index.html
bun run tauri dev
```
