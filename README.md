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
✅ **Sidebar Component** - Browse tables with icons (users, organizations as test tables)
✅ **Tab System** - Open multiple tables, auto-focus existing tabs, close with X button
✅ **Keyboard Shortcuts** - Full keyboard navigation (Cmd/Ctrl+T, Cmd/Ctrl+W, Cmd/Ctrl+[1-9])
✅ **Empty Tabs** - Create blank tabs for custom queries or future features
✅ **Theming** - shadcn-style CSS variables supporting light/dark themes
✅ **Theme Toggle** - Switch themes with sun/moon icon in sidebar
✅ **Responsive Layout** - Flexbox-based layout that adapts to window size
✅ **Dummy Data** - Sample table views for testing the UI

## Features

- **Sidebar Navigation** - Browse available database tables
- **Tab-Based Interface** - Open multiple tables simultaneously, similar to Zed editor
- **Theme Support** - shadcn-style theming with CSS variables for easy theme switching
- **Privacy First** - All data processing happens locally

## UI Components

### Layout Structure

- **Sidebar**: Displays available tables with icons
- **TabBar**: Manages open tabs with close buttons
- **TableView**: Displays table data in a formatted grid
- **EmptyState**: Shown when no tabs are open

### Tab Behavior

- Clicking a table in the sidebar opens it in a new tab
- If the table is already open, it focuses the existing tab
- Tabs can be closed with the X button (appears on hover) or `Cmd/Ctrl + W`
- Active tab is highlighted with a blue underline
- Create empty tabs with `Cmd/Ctrl + T` for custom queries
- Switch between tabs using `Cmd/Ctrl + [1-9]`

### Theming

The app uses shadcn-style CSS variables for theming. Colors are defined in `src/index.css`:

- Light theme (default)
- Dark theme (`.dark` class)

To switch themes, add the `dark` class to the root element:

```typescript
document.documentElement.classList.add('dark');
```

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

## Tech Stack

- **Tauri** - Desktop app framework
- **React 19** - UI framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Utility-first styling
- **Lucide React** - Icon library
- **Vite** - Build tool

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
