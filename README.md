# Daedalus

A local privacy-focused database inspector built with Tauri, React, and TypeScript.

## Quick Start

```bash
# Install dependencies
bun install

# Run in development mode
bun run dev

# Run as Tauri desktop app
bun run tauri dev

# Build for production
bun run build
```

## What's Been Built

✅ **Core UI Layout** - Complete sidebar and tab-based navigation system (like Zed editor)
✅ **Sidebar Component** - Browse tables with icons using shadcn sidebar components
✅ **Tab System** - Open multiple tables, auto-focus existing tabs, close with X button
✅ **Keyboard Shortcuts** - Full keyboard navigation (Cmd/Ctrl+T, Cmd/Ctrl+W, Cmd/Ctrl+[1-9])
✅ **Empty Tabs** - Create blank tabs for custom queries or future features
✅ **Settings Panel** - Dedicated settings tab accessible from sidebar
✅ **Connection Page** - Connect to PostgreSQL databases with test connection feature
✅ **Database Integration** - Full Rust backend with PostgreSQL support
✅ **Live Table Data** - Fetch and display real data from connected databases
✅ **Pagination** - Navigate through large datasets with pagination controls
✅ **Theming System** - 3 beautiful themes with light/dark mode variants
  - Amethyst Haze (Purple-tinted elegance)
  - Solar Dusk (Warm sunset tones)
  - Nature (Fresh green palette)
✅ **Light/Dark Mode** - Separate toggle for light and dark mode
✅ **Theme Toggle** - Quick toggle between light/dark in sidebar
✅ **Responsive Layout** - Flexbox-based layout that adapts to window size
✅ **State Management** - Zustand store for database connection and table management
✅ **Smart Tab Reuse** - Empty tabs are reused when clicking sidebar items
✅ **Smart Numbering** - Untitled tabs numbered based on currently open tabs

## Features

- **PostgreSQL Integration** - Connect to local or remote PostgreSQL databases
- **Live Data Viewing** - Fetch and display real table data with pagination
- **Sidebar Navigation** - Browse available database tables and access settings
- **Tab-Based Interface** - Open multiple tables simultaneously, similar to Zed editor
- **Connection Management** - Test connections before connecting, manage credentials
- **Settings Panel** - Comprehensive settings with theme and mode selection
- **Multiple Themes** - 3 carefully crafted themes from tweakcn, each with light and dark variants
- **Theme System** - OKLCH color space with shadcn-style CSS variables
- **Privacy First** - All data processing happens locally, credentials never leave your machine

## UI Components

### Layout Structure

- **Sidebar**: Displays available database tables fetched from PostgreSQL
- **TabBar**: Manages open tabs with close buttons
- **TableView**: Displays live table data in a formatted grid with pagination
- **ConnectionPage**: Form to connect to PostgreSQL databases
- **EmptyState**: Shown when no tabs are open

### Database Connection

1. Click **Connection** in the sidebar to open the connection form
2. Enter your PostgreSQL credentials:
   - Host (e.g., localhost)
   - Port (default: 5432)
   - Database name
   - Username
   - Password
3. Click **Test Connection** to verify credentials (optional)
4. Click **Connect** to establish the connection
5. Once connected, tables will appear in the sidebar

### Tab Behavior

- **Smart Tab Opening**: 
  - If an empty tab is active, clicking a table/settings/connection will reuse that tab
  - If the table is already open, it focuses the existing tab
  - Otherwise, creates a new tab with live data
- **Untitled Numbering**: Empty tabs are numbered based on currently open tabs
  - Example: Open Untitled 1, 2, 3 → Close Untitled 2 → New tab becomes Untitled 2 (not 4)
  - Always uses the lowest available number
- Tabs can be closed with the X button (appears on hover) or `Cmd/Ctrl + W`
- Active tab is highlighted with a blue underline
- Create empty tabs with `Cmd/Ctrl + T` for custom queries
- Switch between tabs using `Cmd/Ctrl + [1-9]`
- Navigate through large datasets using pagination controls (Previous/Next)

### Settings

Access settings by clicking the Settings button at the bottom of the sidebar.

**Available Settings:**
- **Mode Selection** - Choose between Light and Dark mode
- **Theme Selection** - Choose from 3 beautiful themes:
  - **Amethyst Haze** - Purple-tinted elegance with soft gradients
  - **Solar Dusk** - Warm sunset tones with orange accents
  - **Nature** - Fresh green palette inspired by nature

Each theme includes both light and dark variants that change based on your mode selection.

### Theming

The app uses OKLCH color space with shadcn-style CSS variables. All themes are defined in `src/themes/`:

- 3 pre-built themes with full light/dark variants
- Separate mode (light/dark) and theme selection
- Easy switching via Settings panel
- Settings persist across sessions (localStorage)
- Modern OKLCH colors for vivid, perceptually uniform palettes

#### Theme Variables

All theme colors use HSL values and are defined as CSS variables:
- `--background`, `--foreground`
- `--primary`, `--primary-foreground`
- `--secondary`, `--secondary-foreground`
- `--muted`, `--muted-foreground`
- `--accent`, `--accent-foreground`
- `--card`, `--card-foreground`
- `--border`, `--input`, `--ring`

## Development

```bash
# Install dependencies
bun install

# Run development server
bun run dev

# Build for production
bun run build

# Run Tauri app
bun run tauri dev
```

## Keyboard Shortcuts

- **`⌘/Ctrl + T`** - Create new empty tab
- **`⌘/Ctrl + W`** - Close active tab
- **`⌘/Ctrl + 1-9`** - Switch to tab by number

The app automatically uses `⌘ (Command)` on macOS and `Ctrl` on Windows/Linux.

## Themes

The app includes 3 carefully selected themes from [tweakcn](https://tweakcn.com):

| Theme | Description | Colors |
|-------|-------------|--------|
| Amethyst Haze | Purple-tinted elegance | Purple, Pink, Rose |
| Solar Dusk | Warm sunset tones | Orange, Yellow, Gold |
| Nature | Fresh green palette | Green, Forest, Sage |

Each theme has both light and dark variants. Change theme and mode anytime via **Settings → Appearance**.

## Tech Stack

### Frontend
- **Tauri** - Desktop app framework
- **React 19** - UI framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Utility-first styling
- **shadcn/ui** - UI component library
- **Zustand** - State management
- **Lucide React** - Icon library
- **Vite** - Build tool

### Backend (Rust)
- **tokio** - Async runtime
- **tokio-postgres** - PostgreSQL client
- **serde** - Serialization/deserialization
- **tauri** - Desktop app backend

## Database Support

Currently supports **PostgreSQL** databases with the following features:

- Test connection before connecting
- Fetch table list from `information_schema`
- View table data with pagination (100 rows per page)
- Support for multiple schemas
- NULL value handling
- Type-safe data serialization

Future support planned for MySQL, SQLite, and other databases.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
