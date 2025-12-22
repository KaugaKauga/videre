# Daedalus Workflows Guide

Visual guide showing common workflows and how keyboard shortcuts enhance productivity.

## Workflow 1: Opening and Comparing Multiple Tables

### Without Keyboard Shortcuts (Mouse Only)

```
1. Click "users" in sidebar
   → Users table opens in Tab 1

2. Click "organizations" in sidebar
   → Organizations table opens in Tab 2

3. Click Tab 1 to view users
   → View users data

4. Click Tab 2 to view organizations
   → View organizations data

5. Hover over Tab 2 and click X
   → Organizations tab closes
```

### With Keyboard Shortcuts (Hybrid Approach)

```
1. Click "users" in sidebar
   → Users table opens in Tab 1

2. Click "organizations" in sidebar
   → Organizations table opens in Tab 2

3. Press Cmd/Ctrl + 1
   → Instantly switch to users (Tab 1)

4. Press Cmd/Ctrl + 2
   → Instantly switch to organizations (Tab 2)

5. Press Cmd/Ctrl + W
   → Close current tab (organizations)
```

**Time Saved**: ~40% faster tab switching, no need to aim for tabs

---

## Workflow 2: Creating a Workspace for Analysis

### Scenario: Analyzing relationships between users and organizations

```
Step 1: Open relevant tables
┌─────────────────────────────────┐
│ Click "users" in sidebar        │
│ Click "organizations" in sidebar│
└─────────────────────────────────┘
         ↓
┌─────────────────────────────────┐
│ Tab 1: users                    │
│ Tab 2: organizations            │
└─────────────────────────────────┘

Step 2: Create empty tab for notes/queries
┌─────────────────────────────────┐
│ Press Cmd/Ctrl + T              │
└─────────────────────────────────┘
         ↓
┌─────────────────────────────────┐
│ Tab 1: users                    │
│ Tab 2: organizations            │
│ Tab 3: Untitled 1 (active)      │
└─────────────────────────────────┘

Step 3: Navigate between tabs
┌─────────────────────────────────┐
│ Cmd/Ctrl + 1 → View users       │
│ Cmd/Ctrl + 2 → View orgs        │
│ Cmd/Ctrl + 3 → View notes       │
└─────────────────────────────────┘

Step 4: Clean up when done
┌─────────────────────────────────┐
│ Cmd/Ctrl + 3 (focus notes tab)  │
│ Cmd/Ctrl + W (close it)         │
└─────────────────────────────────┘
```

---

## Workflow 3: Power User - Rapid Tab Management

### Creating a Multi-Tab Workspace in Seconds

```
Starting State:
┌──────────────┐
│ Empty State  │
│ No tabs open │
└──────────────┘

Action Sequence:
Cmd/Ctrl + T  →  Cmd/Ctrl + T  →  Cmd/Ctrl + T
      ↓                ↓                ↓
  Untitled 1      Untitled 2       Untitled 3

Click "users" → Click "organizations"
      ↓                  ↓
   users tab         orgs tab

Final State:
┌─────────────────────────────────────────────┐
│ Tab 1: Untitled 1                           │
│ Tab 2: Untitled 2                           │
│ Tab 3: Untitled 3                           │
│ Tab 4: users                                │
│ Tab 5: organizations                        │
└─────────────────────────────────────────────┘

Quick Navigation:
Cmd/Ctrl + 1  →  Notes/queries
Cmd/Ctrl + 4  →  Users data
Cmd/Ctrl + 5  →  Organizations data
```

---

## Workflow 4: Tab Organization Best Practices

### Organizing Tabs for Optimal Productivity

```
Recommended Layout (Positions 1-9):

Position 1-3: Empty tabs for queries/notes
┌─────────────────────────────────┐
│ 1: Query Builder                │
│ 2: Notes                        │
│ 3: Scratch Space                │
└─────────────────────────────────┘

Position 4-9: Frequently accessed tables
┌─────────────────────────────────┐
│ 4: users                        │
│ 5: organizations                │
│ 6: transactions                 │
│ 7: products                     │
│ 8-9: As needed                  │
└─────────────────────────────────┘

Quick Access Pattern:
- Cmd/Ctrl + 1 = Your main work area
- Cmd/Ctrl + 4 = Primary data source
- Cmd/Ctrl + 5 = Secondary data source
```

---

## Workflow 5: Cleaning Up Your Workspace

### Efficient Tab Cleanup

```
Before Cleanup (9 tabs open):
┌─────────────────────────────────────────────┐
│ 1  2  3  4  5  6  7  8  9                   │
│ ●  ○  ○  ○  ○  ○  ○  ○  ○                   │
└─────────────────────────────────────────────┘

Method 1: Close from right to left
Cmd/Ctrl + 9  →  Cmd/Ctrl + W  →  Close Tab 9
Cmd/Ctrl + 8  →  Cmd/Ctrl + W  →  Close Tab 8
Cmd/Ctrl + 7  →  Cmd/Ctrl + W  →  Close Tab 7
...continue as needed

Method 2: Close sequentially
Cmd/Ctrl + 2  →  Cmd/Ctrl + W  →  Close Tab 2
(automatic focus to last tab)
Cmd/Ctrl + W  →  Close current tab
Cmd/Ctrl + W  →  Close current tab
...repeat as needed

After Cleanup (3 tabs remaining):
┌─────────────────────────────────────────────┐
│ 1  2  3                                     │
│ ●  ○  ○                                     │
└─────────────────────────────────────────────┘
```

---

## Workflow 6: Research & Comparison

### Comparing Data Across Multiple Tables

```
Research Task: Find user patterns across tables

Step 1: Set up tabs
┌────────────────────────────────────────┐
│ Cmd/Ctrl + T → Create "Research" tab  │
│ Click "users" → Open users table       │
│ Click "organizations" → Open orgs table│
└────────────────────────────────────────┘

Step 2: Navigate pattern
┌────────────────────────────────────────┐
│ Cmd/Ctrl + 1 → Write query/notes       │
│ Cmd/Ctrl + 2 → Check users data        │
│ Cmd/Ctrl + 3 → Check orgs data         │
│ Cmd/Ctrl + 1 → Back to notes           │
└────────────────────────────────────────┘

Visual Layout:
┌─────────────┬─────────────┬─────────────┐
│   Tab 1     │   Tab 2     │   Tab 3     │
│  Research   │   users     │    orgs     │
├─────────────┼─────────────┼─────────────┤
│ Notes:      │ ID | Name   │ ID | Name   │
│ - Pattern 1 │ 1  | John   │ 1  | Acme   │
│ - Pattern 2 │ 2  | Jane   │ 2  | Corp   │
│ - Findings  │ 3  | Bob    │ 3  | Inc    │
└─────────────┴─────────────┴─────────────┘

Keyboard Flow:
1→2→3→1→2→3→1 (rapid switching)
```

---

## Workflow 7: Handling Tab Duplicates

### Smart Tab Management

```
Scenario: User clicks "users" twice

First Click:
┌─────────────────────────────────┐
│ Click "users"                   │
└─────────────────────────────────┘
         ↓
┌─────────────────────────────────┐
│ Tab 1: users (created & focused)│
└─────────────────────────────────┘

Second Click:
┌─────────────────────────────────┐
│ Click "users" again             │
└─────────────────────────────────┘
         ↓
┌─────────────────────────────────┐
│ Tab 1: users (focused, not      │
│              duplicated!)       │
└─────────────────────────────────┘

Result: No duplicate tabs created!
✓ Prevents clutter
✓ Focuses existing tab
✓ Maintains clean workspace
```

---

## Workflow 8: Empty Tab Use Cases

### Creative Uses for Empty Tabs

```
Use Case 1: SQL Query Builder
┌─────────────────────────────────┐
│ Cmd/Ctrl + T                    │
│ → "Untitled 1"                  │
│ → Write and test SQL queries    │
└─────────────────────────────────┘

Use Case 2: Analysis Notes
┌─────────────────────────────────┐
│ Cmd/Ctrl + T                    │
│ → "Untitled 2"                  │
│ → Document findings             │
└─────────────────────────────────┘

Use Case 3: Comparison Matrix
┌─────────────────────────────────┐
│ Cmd/Ctrl + T                    │
│ → "Untitled 3"                  │
│ → Track comparisons             │
└─────────────────────────────────┘

Use Case 4: Temporary Workspace
┌─────────────────────────────────┐
│ Cmd/Ctrl + T                    │
│ → Quick calculations            │
│ → Cmd/Ctrl + W to close         │
└─────────────────────────────────┘
```

---

## Keyboard Shortcuts Cheat Sheet

### Quick Reference

```
┌──────────────────────────────────────────────┐
│          TAB MANAGEMENT                      │
├──────────────────────────────────────────────┤
│ Cmd/Ctrl + T     New empty tab               │
│ Cmd/Ctrl + W     Close active tab            │
│ Cmd/Ctrl + 1-9   Jump to tab (position 1-9)  │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│          COMMON PATTERNS                     │
├──────────────────────────────────────────────┤
│ T + T + T        Create 3 empty tabs quickly │
│ 1, 2, 3, ...     Navigate between tabs       │
│ W                Close current work          │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│          PLATFORM-SPECIFIC                   │
├──────────────────────────────────────────────┤
│ macOS            Use ⌘ (Command) key         │
│ Windows/Linux    Use Ctrl key               │
└──────────────────────────────────────────────┘
```

---

## Productivity Tips

### Maximize Efficiency

1. **Keep important tabs in positions 1-9**
   - Instant access via Cmd/Ctrl + [number]
   - No need to click or search

2. **Use empty tabs as scratchpads**
   - Quick note-taking
   - Query planning
   - Temporary calculations

3. **Close tabs frequently**
   - Cmd/Ctrl + W is faster than mouse
   - Keeps workspace clean
   - Improves focus

4. **Develop muscle memory**
   - Practice common patterns
   - Create your own workflows
   - Speed increases with use

5. **Combine mouse and keyboard**
   - Click to open tables (sidebar)
   - Keyboard to navigate (tabs)
   - Best of both worlds

---

## Workflow Patterns Summary

```
Opening Pattern:
Click → Click → Cmd/Ctrl+T → Cmd/Ctrl+T
(Open tables, create empty tabs)

Navigation Pattern:
Cmd/Ctrl+1 → Cmd/Ctrl+2 → Cmd/Ctrl+3
(Switch between tabs rapidly)

Cleanup Pattern:
Cmd/Ctrl+W → Cmd/Ctrl+W → Cmd/Ctrl+W
(Close tabs sequentially)

Research Pattern:
Cmd/Ctrl+1 (notes) ↔ Cmd/Ctrl+2 (data) ↔ Cmd/Ctrl+3 (data)
(Bounce between research and sources)
```

---

## Advanced Tips

### Power User Techniques

1. **Tab Bookmarking Pattern**
   ```
   Position 1: Always your main workspace
   Position 2-3: Most frequently used tables
   Position 4-6: Secondary tables
   Position 7-9: Temporary tabs
   ```

2. **Rapid Context Switching**
   ```
   One finger on Cmd/Ctrl
   Other hand on number keys
   Switch tabs without looking
   ```

3. **Efficient Cleanup Strategy**
   ```
   Work from right to left (9→8→7...)
   Or close all with rapid Cmd/Ctrl+W
   Keep essential tabs (1-3)
   ```

4. **Workspace Templates**
   ```
   Template A: Research
   - Tab 1: Notes
   - Tab 2-4: Data sources
   
   Template B: Development
   - Tab 1: Query builder
   - Tab 2-3: Test tables
   ```

---

**Last Updated**: January 2024  
**Version**: 0.1.0