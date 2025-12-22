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
✅ **Settings Panel** - Dedicated settings tab accessible from sidebar
✅ **Theming System** - 3 beautiful themes with light/dark mode variants
  - Amethyst Haze (Purple-tinted elegance)
  - Solar Dusk (Warm sunset tones)
  - Nature (Fresh green palette)
✅ **Light/Dark Mode** - Separate toggle for light and dark mode
✅ **Theme Toggle** - Quick toggle between light/dark in sidebar
✅ **Responsive Layout** - Flexbox-based layout that adapts to window size
✅ **Dummy Data** - Sample table views for testing the UI

## Features

- **Sidebar Navigation** - Browse available database tables and access settings
- **Tab-Based Interface** - Open multiple tables simultaneously, similar to Zed editor
- **Settings Panel** - Comprehensive settings with theme and mode selection
- **Multiple Themes** - 3 carefully crafted themes from tweakcn, each with light and dark variants
- **Theme System** - OKLCH color space with shadcn-style CSS variables
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

- **Tauri** - Desktop app framework
- **React 19** - UI framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Utility-first styling
- **Lucide React** - Icon library
- **Vite** - Build tool

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
