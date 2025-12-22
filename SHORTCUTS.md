# Keyboard Shortcuts Quick Reference

A comprehensive guide to all keyboard shortcuts in Daedalus.

## Tab Management

| Shortcut | Action | Description |
|----------|--------|-------------|
| `⌘/Ctrl + T` | New Tab | Creates a new empty tab for custom queries or future features |
| `⌘/Ctrl + W` | Close Tab | Closes the currently active tab |
| `⌘/Ctrl + 1` | Jump to Tab 1 | Switches focus to the first tab |
| `⌘/Ctrl + 2` | Jump to Tab 2 | Switches focus to the second tab |
| `⌘/Ctrl + 3` | Jump to Tab 3 | Switches focus to the third tab |
| `⌘/Ctrl + 4` | Jump to Tab 4 | Switches focus to the fourth tab |
| `⌘/Ctrl + 5` | Jump to Tab 5 | Switches focus to the fifth tab |
| `⌘/Ctrl + 6` | Jump to Tab 6 | Switches focus to the sixth tab |
| `⌘/Ctrl + 7` | Jump to Tab 7 | Switches focus to the seventh tab |
| `⌘/Ctrl + 8` | Jump to Tab 8 | Switches focus to the eighth tab |
| `⌘/Ctrl + 9` | Jump to Tab 9 | Switches focus to the ninth tab |

## Platform-Specific Keys

### macOS
- All shortcuts use the `⌘ (Command)` key
- Example: `⌘ + T` to create a new tab

### Windows & Linux
- All shortcuts use the `Ctrl` key
- Example: `Ctrl + T` to create a new tab

The application automatically detects your operating system and uses the appropriate modifier key.

## Workflow Examples

### Opening and Managing Multiple Tables

1. **Open a table**: Click on "users" in the sidebar
   - Opens in a new tab
2. **Open another table**: Click on "organizations" in the sidebar
   - Opens in another new tab
3. **Switch between them**:
   - `⌘/Ctrl + 1` for users (first tab)
   - `⌘/Ctrl + 2` for organizations (second tab)
4. **Close a tab**: `⌘/Ctrl + W` when focused on the tab you want to close

### Working with Empty Tabs

1. **Create empty tab**: Press `⌘/Ctrl + T`
   - A new "Untitled 1" tab appears
2. **Create another**: Press `⌘/Ctrl + T` again
   - A new "Untitled 2" tab appears
3. **Switch between them**: Use `⌘/Ctrl + [number]`
4. **Close when done**: Press `⌘/Ctrl + W`

## Tips & Tricks

### Quick Navigation
- Keep frequently accessed tables in the first 9 tab positions for quick access via `⌘/Ctrl + [1-9]`
- Empty tabs are useful for planning queries or taking notes while browsing tables

### Efficient Workflow
- Use `⌘/Ctrl + T` to create a workspace for complex queries
- Open related tables in sequential tabs for easy comparison
- Use `⌘/Ctrl + W` to quickly close tabs you're done with

### Tab Behavior
- Opening the same table twice will focus the existing tab instead of creating a duplicate
- Closing the active tab automatically focuses the last tab in the list
- If you close all tabs, you'll see the welcome screen

## Planned Future Shortcuts

The following shortcuts are planned for future releases:

| Shortcut | Action | Status |
|----------|--------|--------|
| `⌘/Ctrl + Tab` | Next Tab | Planned |
| `⌘/Ctrl + Shift + Tab` | Previous Tab | Planned |
| `⌘/Ctrl + F` | Search in Table | Planned |
| `⌘/Ctrl + ,` | Open Settings | Planned |
| `⌘/Ctrl + K` | Command Palette | Planned |
| `⌘/Ctrl + N` | New Window | Planned |
| `⌘/Ctrl + Shift + F` | Global Search | Planned |

## Accessibility

All keyboard shortcuts are designed to work alongside screen readers and other accessibility tools:

- Shortcuts don't interfere with browser accessibility features
- Visual indicators show which tab is active
- Keyboard focus is properly managed when switching tabs
- All actions have visible UI equivalents (buttons, menu items)

## Customization

Currently, keyboard shortcuts are not customizable. This feature may be added in a future release based on user feedback.

If you'd like to request a specific shortcut or suggest changes, please open an issue on the project repository.

## Technical Details

- All shortcuts use `preventDefault()` to avoid conflicts with browser defaults
- Platform detection uses `navigator.platform` for accurate OS detection
- Shortcuts are registered using React hooks for proper cleanup
- Event listeners are automatically removed when components unmount

## Troubleshooting

### Shortcuts Not Working

1. **Check focus**: Ensure the Daedalus window has focus
2. **Browser conflicts**: Some browsers may override certain shortcuts
3. **Operating system**: Verify you're using the correct modifier key for your OS
4. **Tauri app**: In the desktop app, all shortcuts should work without conflicts

### Conflicts with Browser Shortcuts

If you encounter conflicts:

- **Browser**: Use the Tauri desktop app for the best experience
- **Tauri app**: All shortcuts work as expected with no browser interference

### Missing Modifier Keys

- **Windows/Linux**: Ensure Ctrl key is working properly
- **macOS**: Ensure Command (⌘) key is working properly
- **Test**: Try using the shortcut in other applications to verify hardware

---

**Last Updated**: January 2024  
**Version**: 0.1.0