# Daedalus Architecture

## Overview

Daedalus is a local privacy-focused database inspector built with a clean, modular architecture using Tauri, React, and TypeScript.

## Project Structure

```
Daedalus/
├── src/
│   ├── components/           # React components
│   │   ├── Sidebar.tsx      # Left sidebar with table list
│   │   ├── TabBar.tsx       # Top tab navigation
│   │   ├── TableView.tsx    # Main content area for tables
│   │   ├── EmptyState.tsx   # Empty state when no tabs open
│   │   ├── EmptyTab.tsx     # Empty tab content
│   │   └── index.ts         # Component exports
│   ├── hooks/               # Custom React hooks
│   │   ├── useKeyboardShortcuts.ts  # Keyboard shortcut management
│   │   └── index.ts         # Hook exports
│   ├── lib/                 # Utilities
│   │   └── theme.ts         # Theme management utilities
│   ├── App.tsx              # Main application component
│   ├── main.tsx             # React entry point
│   └── index.css            # Global styles + theme variables
├── src-tauri/               # Tauri backend (Rust)
├── tailwind.config.js       # Tailwind configuration
└── vite.config.ts           # Vite build configuration
```

## Component Architecture

### App Component (Main Container)

The `App` component is the root of the application and manages:

- **State Management**
  - `tabs`: Array of open tabs
  - `activeTabId`: Currently focused tab ID
  - `tabCounter`: Counter for naming empty tabs
  
- **Event Handlers**
  - `handleTableClick`: Opens or focuses a table tab
  - `handleTabClick`: Switches between tabs
  - `handleTabClose`: Closes a tab and manages focus
  - `handleNewEmptyTab`: Creates a new empty tab
  - `handleCloseActiveTab`: Closes the currently active tab
  - `handleSwitchToTab`: Switches to a tab by index

- **Keyboard Shortcuts**
  - `Cmd/Ctrl + T`: Create new empty tab
  - `Cmd/Ctrl + W`: Close active tab
  - `Cmd/Ctrl + 1-9`: Switch to tab by index (1-9)

**Layout Structure:**
```
┌─────────────────────────────────────────┐
│ App (flex container)                    │
│ ┌──────────┬──────────────────────────┐ │
│ │          │ TabBar                   │ │
│ │ Sidebar  ├──────────────────────────┤ │
│ │          │ TableView / EmptyState   │ │
│ │          │                          │ │
│ └──────────┴──────────────────────────┘ │
└─────────────────────────────────────────┘
```

### Sidebar Component

**Purpose:** Display available database tables and handle navigation

**Props:**
- `onTableClick: (tableName: string) => void`

**Features:**
- Header with database icon and app name
- Scrollable list of tables
- Hover states for table items
- Uses shadcn-style colors for consistent theming

**Data:**
Currently uses hardcoded dummy data:
```typescript
const tables = [
  { name: "users", icon: Table },
  { name: "organizations", icon: Table },
];
```

### TabBar Component

**Purpose:** Manage and display open tabs with close functionality

**Props:**
- `tabs: Tab[]` - Array of open tabs
- `activeTabId: string | null` - ID of currently active tab
- `onTabClick: (tabId: string) => void` - Tab click handler
- `onTabClose: (tabId: string) => void` - Tab close handler

**Features:**
- Active tab highlighting with bottom border
- Close button (appears on hover)
- Horizontal scroll for many tabs
- Click-through prevention on close button

**Tab Interface:**
```typescript
interface Tab {
  id: string;         // Unique identifier
  label: string;      // Display name
  type: "table" | "empty";  // Tab type
  tableName?: string; // Table name (for table tabs)
}
```

**Tab Types:**
- `table`: Displays database table content
- `empty`: Blank tab for custom queries or future features

### TableView Component

**Purpose:** Display table data in a structured format

**Props:**
- `tableName: string` - Name of the table to display

**Features:**
- Responsive table layout
- Header row with column names
- Dummy data for demonstration
- Hover effects on rows
- Currently displays: ID, Name, Created At

**Future Enhancements:**
- Dynamic column generation
- Real database connection
- Pagination
- Sorting and filtering
- Cell editing

### EmptyState Component

**Purpose:** Show placeholder when no tabs are open

**Features:**
- Centered layout
- Database icon
- Instructional text
- Uses muted colors for subtle appearance

### EmptyTab Component

**Purpose:** Display content for empty tabs created with Cmd/Ctrl+T

**Features:**
- Centered layout with file icon
- Helpful information about the tab
- Keyboard shortcuts guide
- Placeholder for future query functionality

## State Management

### Tab Management Logic

1. **Opening a Table Tab:**
   - Check if tab for that table already exists
   - If exists: Focus existing tab
   - If new: Create tab with type "table" and add to array
   - Tab ID includes table name and timestamp for uniqueness

2. **Creating an Empty Tab:**
   - Generate unique ID with counter and timestamp
   - Create tab with type "empty"
   - Label as "Untitled [counter]"
   - Increment tab counter for next empty tab

3. **Closing a Tab:**
   - Remove from tabs array
   - If closed tab was active:
     - Focus last tab in array
     - If no tabs left, set activeTabId to null

4. **Tab Focus:**
   - Only one tab can be active at a time
   - Active tab determined by `activeTabId` state
   - Can switch focus via clicks or keyboard shortcuts

## Keyboard Shortcuts System

### Hook Architecture

**`useKeyboardShortcuts`** - Custom React hook for managing keyboard shortcuts

**Features:**
- Platform detection (macOS vs Windows/Linux)
- Automatic modifier key handling (⌘ on Mac, Ctrl elsewhere)
- Support for modifier combinations (Ctrl/Cmd, Shift, Alt)
- Automatic event cleanup on unmount

**Interface:**
```typescript
interface KeyboardShortcut {
  key: string;              // The key to listen for
  ctrlOrCmd?: boolean;      // Requires Cmd (Mac) or Ctrl (Win/Linux)
  shift?: boolean;          // Requires Shift key
  alt?: boolean;            // Requires Alt/Option key
  handler: (event: KeyboardEvent) => void;  // Callback function
  preventDefault?: boolean; // Prevent default browser behavior
}
```

**Usage Example:**
```typescript
const shortcuts = useMemo(() => [
  {
    key: 't',
    ctrlOrCmd: true,
    handler: () => handleNewEmptyTab(),
  },
  {
    key: 'w',
    ctrlOrCmd: true,
    handler: () => handleCloseActiveTab(),
  },
], [dependencies]);

useKeyboardShortcuts(shortcuts);
```

### Platform Detection

The hook automatically detects the user's platform:
- Checks `navigator.platform` for Mac detection
- Uses `event.metaKey` on macOS (⌘ Command key)
- Uses `event.ctrlKey` on Windows/Linux (Ctrl key)

### Implemented Shortcuts

| Shortcut | Action | Handler |
|----------|--------|---------|
| `Cmd/Ctrl + T` | New empty tab | `handleNewEmptyTab()` |
| `Cmd/Ctrl + W` | Close active tab | `handleCloseActiveTab()` |
| `Cmd/Ctrl + 1-9` | Switch to tab N | `handleSwitchToTab(N-1)` |

## Theming System

### CSS Variable Architecture

Based on shadcn/ui standards, using HSL color space for flexibility.

**Theme Structure:**
```css
:root {
  /* Light theme variables */
}

.dark {
  /* Dark theme variables */
}
```

**Color Categories:**
- **Layout**: background, foreground, border
- **Components**: card, popover, input
- **Semantic**: primary, secondary, muted, accent, destructive
- **Interactive**: ring (focus states)

**Usage in Components:**
```typescript
className="bg-background text-foreground"
className="bg-card border-border"
className="hover:bg-accent hover:text-accent-foreground"
```

### Theme Switching

To implement theme switching, toggle the `dark` class on the root element:

```typescript
// Enable dark theme
document.documentElement.classList.add('dark');

// Enable light theme
document.documentElement.classList.remove('dark');
```

## Data Flow

### User Clicks Table in Sidebar
```
User Action (Click Table)
    ↓
Sidebar → onTableClick(tableName)
    ↓
App → handleTableClick()
    ↓
Check if table tab exists
    ↓
  Yes: Focus existing tab
  No: Create new table tab
    ↓
Update state (tabs, activeTabId)
    ↓
TabBar renders updated tabs
    ↓
TableView renders active table
```

### User Presses Cmd/Ctrl+T
```
User Presses Cmd/Ctrl+T
    ↓
useKeyboardShortcuts detects event
    ↓
Prevents default browser behavior
    ↓
App → handleNewEmptyTab()
    ↓
Increment tabCounter
    ↓
Create new empty tab
    ↓
Update state (tabs, activeTabId, tabCounter)
    ↓
TabBar renders updated tabs
    ↓
EmptyTab renders content
```

### User Presses Cmd/Ctrl+W
```
User Presses Cmd/Ctrl+W
    ↓
useKeyboardShortcuts detects event
    ↓
Prevents default browser behavior
    ↓
App → handleCloseActiveTab()
    ↓
App → handleTabClose(activeTabId)
    ↓
Remove tab from array
    ↓
Focus last remaining tab or show empty state
    ↓
Update state (tabs, activeTabId)
    ↓
Re-render UI
```

## Future Architecture Considerations

### Database Integration Layer

```
┌─────────────────────────────────────┐
│ React Components (UI)               │
├─────────────────────────────────────┤
│ State Management (React Context?)   │
├─────────────────────────────────────┤
│ Database Service Layer              │
├─────────────────────────────────────┤
│ Tauri Commands (IPC)                │
├─────────────────────────────────────┤
│ Rust Backend (SQL queries)          │
├─────────────────────────────────────┤
│ Local Database Files                │
└─────────────────────────────────────┘
```

### Recommended Patterns

1. **Context API or Zustand** for global state management
2. **React Query** for database query caching
3. **Tauri Commands** for secure database operations
4. **Virtual Scrolling** for large datasets
5. **Web Workers** for heavy data processing
6. **Command Palette** for advanced keyboard navigation

### Keyboard Shortcuts Extensibility

To add new shortcuts, update the shortcuts array in `App.tsx`:

```typescript
const shortcuts = useMemo(() => [
  // Existing shortcuts...
  {
    key: 'f',
    ctrlOrCmd: true,
    handler: () => handleSearch(),
  },
], [dependencies]);
```

## Performance Considerations

- Tab state is kept minimal (id, label, type, optional tableName)
- Components are pure and memoizable
- Virtual DOM handles efficient re-renders
- Tailwind CSS purges unused styles in production
- Keyboard shortcuts use `useMemo` to prevent unnecessary re-registrations
- Event listeners properly cleaned up on unmount

## Security & Privacy

- All database operations happen locally
- No external API calls
- No telemetry or tracking
- Tauri provides sandboxed environment
- User data never leaves the device