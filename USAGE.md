# Daedalus Usage Guide

## Getting Started

### Running the Application

```bash
# Development mode
bun run dev

# Tauri desktop app (development)
bun run tauri dev

# Build for production
bun run build
```

## Using the Interface

### Navigation

1. **Opening Tables**
   - Click on any table name in the left sidebar (e.g., "users" or "organizations")
   - The table will open in a new tab in the main content area
   - If the table is already open, clicking it again will focus that tab

2. **Managing Tabs**
   - **Switch Tabs**: Click on any tab in the tab bar at the top
   - **Close Tabs**: Hover over a tab and click the X button that appears
   - The active tab is highlighted with a blue underline

3. **Viewing Data**
   - Once a table is open, you'll see the data displayed in a grid format
   - Currently shows dummy data with columns: ID, Name, Created At
   - Hover over rows to see the hover effect

### Theme Switching

The app supports both light and dark themes with automatic system preference detection.

#### Using the Theme Toggle

1. **Manual Toggle**
   - Click the sun/moon icon in the top-left corner of the sidebar
   - The theme will immediately switch
   - Your preference is saved in localStorage

2. **System Theme**
   - By default, the app follows your system's color scheme preference
   - If you manually toggle the theme, it will override the system preference
   - The app will remember your choice across sessions

#### Programmatic Theme Control

```typescript
import { setTheme, toggleTheme, getStoredTheme } from "./lib/theme";

// Set a specific theme
setTheme("dark");  // Options: "light", "dark", "system"

// Toggle between light and dark
toggleTheme();

// Get current theme
const currentTheme = getStoredTheme();
```

## Keyboard Shortcuts

The following keyboard shortcuts are available:

### Tab Management

- **`Cmd/Ctrl + T`** - Create a new empty tab
  - Opens a blank tab that can be used for custom queries or future features
  - Each new tab is numbered sequentially (Untitled 1, Untitled 2, etc.)

- **`Cmd/Ctrl + W`** - Close the current active tab
  - Closes the focused tab
  - Automatically focuses the last tab in the list
  - If no tabs remain, shows the empty state

- **`Cmd/Ctrl + [1-9]`** - Jump to specific tab by number
  - `Cmd/Ctrl + 1` - Switch to first tab
  - `Cmd/Ctrl + 2` - Switch to second tab
  - And so on up to the 9th tab
  - Works with both table tabs and empty tabs

### Platform-Specific Keys

- **macOS**: Uses `⌘ (Command)` key
- **Windows/Linux**: Uses `Ctrl` key

The app automatically detects your platform and uses the appropriate modifier key.

### Future Enhancements

The following shortcuts are planned for future releases:

- `Cmd/Ctrl + Tab` - Switch to next tab
- `Cmd/Ctrl + Shift + Tab` - Switch to previous tab
- `Cmd/Ctrl + ,` - Open settings
- `Cmd/Ctrl + F` - Search within table

## Customizing Themes

### Creating a Custom Theme

Themes are defined using CSS variables in `src/index.css`. To create a custom theme:

1. Add a new theme class:

```css
.custom-theme {
  --background: 240 10% 3.9%;
  --foreground: 0 0% 98%;
  --primary: 142 76% 36%;
  --primary-foreground: 144 61% 20%;
  /* ... add all other variables */
}
```

2. Apply the theme by adding the class to the document:

```typescript
document.documentElement.classList.add('custom-theme');
```

### Color Format

All colors use HSL (Hue, Saturation, Lightness) format without the `hsl()` wrapper:

- **Format**: `hue saturation% lightness%`
- **Example**: `221.2 83.2% 53.3%` (blue)
- **Usage**: `hsl(var(--primary))` in CSS

This format allows Tailwind to add opacity modifiers:
```typescript
className="bg-primary/50"  // 50% opacity
```

## Component Overview

### Sidebar
- **Location**: Left side of the screen
- **Width**: 256px (16rem)
- **Content**: 
  - App header with theme toggle
  - List of available tables

### Tab Bar
- **Location**: Top of the main content area
- **Height**: 40px
- **Features**:
  - Horizontal scroll for many tabs
  - Close buttons on hover
  - Active tab indicator

### Table View
- **Location**: Main content area
- **Features**:
  - Responsive table layout
  - Hover effects on rows
  - Header with table name

### Empty State
- **When Shown**: When no tabs are open
- **Purpose**: Guide users to select a table

## Tips & Tricks

1. **Multiple Tables**: You can have multiple tables open simultaneously. This is useful for comparing data across tables.

2. **Tab Persistence**: Currently, tabs are cleared on app restart. This is by design for privacy, but can be changed if needed.

3. **Performance**: The app is optimized for local databases. All operations happen on your machine, ensuring fast performance and complete privacy.

4. **Theme Persistence**: Your theme choice is saved in localStorage and will persist across app restarts.

5. **Responsive Design**: While optimized for desktop use via Tauri, the UI is fully responsive and will work in different window sizes.

## Troubleshooting

### Theme Not Switching

1. Check browser console for errors
2. Ensure localStorage is enabled
3. Try clearing localStorage: `localStorage.clear()`
4. Restart the application

### Tabs Not Opening

1. Check the browser console for errors
2. Ensure the table name is valid
3. Try closing other tabs first

### Styling Issues

1. Clear your browser cache
2. Run `bun run build` to rebuild
3. Check that Tailwind CSS is properly configured
4. Verify all CSS variables are defined in `index.css`

## Next Steps

- Connect to a real database
- Add pagination for large datasets
- Implement search and filtering
- Add data editing capabilities
- Export data functionality
- Custom SQL query interface