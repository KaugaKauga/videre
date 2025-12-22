# Themes Guide

Daedalus includes 5 carefully curated themes from [tweakcn](https://tweakcn.com), each with full light and dark variants.

## Available Themes

### 1. Light (Default)
**Classic light theme with clean blues**

- **Best for**: All-purpose use, daytime work
- **Primary Color**: Blue (#3b82f6)
- **Background**: Pure white
- **Accent**: Light gray/blue

Perfect for general use with high contrast and readability.

---

### 2. Dark
**Classic dark theme with deep blues**

- **Best for**: Low-light environments, night work
- **Primary Color**: Bright blue (#60a5fa)
- **Background**: Dark blue-gray
- **Accent**: Medium blue-gray

Easy on the eyes in dark environments while maintaining good contrast.

---

### 3. Amethyst Haze
**Purple-tinted elegance with soft gradients**

- **Best for**: Creative work, softer aesthetic
- **Primary Color**: Purple (#a855f7)
- **Background**: 
  - Light: Soft purple-white
  - Dark: Deep purple-gray
- **Accent**: Pink/Rose tones

A sophisticated theme with purple hues that's easy on the eyes and aesthetically pleasing.

**Features:**
- Soft purple tones throughout
- Pink accent colors
- Gentle on the eyes
- Great for extended use

---

### 4. Solar Dusk
**Warm sunset tones with orange accents**

- **Best for**: Comfortable atmosphere, warm preference
- **Primary Color**: Orange (#f97316)
- **Background**:
  - Light: Warm cream/beige
  - Dark: Deep warm gray
- **Accent**: Yellow/Gold tones

Inspired by sunset colors, this theme provides a warm, inviting workspace.

**Features:**
- Warm orange primary colors
- Sunset-inspired palette
- Comfortable for long sessions
- Unique aesthetic

---

### 5. Nature
**Fresh green palette inspired by nature**

- **Best for**: Calming work environment, nature lovers
- **Primary Color**: Green (#22c55e)
- **Background**:
  - Light: Soft green-white
  - Dark: Deep forest green
- **Accent**: Bright green

A refreshing theme inspired by nature with various shades of green.

**Features:**
- Calming green tones
- Nature-inspired palette
- Easy on the eyes
- Fresh, clean look

---

## How to Change Themes

### Via Settings Panel

1. Click **Settings** at the bottom of the sidebar
2. Navigate to the **Appearance** section
3. Click on any theme card to apply it
4. The theme switches instantly

### Via Keyboard

1. Press `⌘/Ctrl + T` to create a new tab
2. Click **Settings** in the sidebar
3. Select your preferred theme

### Programmatically

```typescript
import { setTheme } from "./lib/theme";

// Set a specific theme
setTheme("amethyst-haze");
setTheme("solar-dusk");
setTheme("nature");
setTheme("light");
setTheme("dark");
```

---

## Theme Persistence

Your theme choice is automatically saved to `localStorage` and will persist across:
- App restarts
- Browser sessions
- Tab closures

The theme is applied immediately on app load.

---

## Light/Dark Variants

All themes include both light and dark variants:

- **Light variant**: Bright backgrounds, dark text
- **Dark variant**: Dark backgrounds, light text

The variant is determined by your theme selection. Each theme (except "Light" and "Dark") has its own unique light and dark color palettes.

---

## Theme Architecture

### CSS Variables

All themes use CSS custom properties (variables) for consistent styling:

```css
--background
--foreground
--card
--card-foreground
--primary
--primary-foreground
--secondary
--secondary-foreground
--muted
--muted-foreground
--accent
--accent-foreground
--destructive
--destructive-foreground
--border
--input
--ring
```

### Theme Classes

Themes are applied by adding a class to the root element:

- `.amethyst-haze` - Amethyst Haze theme
- `.solar-dusk` - Solar Dusk theme
- `.nature` - Nature theme
- `.dark` - Dark theme (built-in)
- No class - Light theme (default)

### Dark Variants

Dark variants are triggered by combining classes:

```css
.amethyst-haze.dark { /* Amethyst Haze dark variant */ }
.solar-dusk.dark { /* Solar Dusk dark variant */ }
.nature.dark { /* Nature dark variant */ }
```

---

## Creating Custom Themes

Want to create your own theme? Follow these steps:

### 1. Define CSS Variables

Add a new class to `src/themes.css`:

```css
.my-custom-theme {
  --background: 0 0% 100%;
  --foreground: 240 10% 4%;
  --primary: 200 100% 50%;
  /* ... define all required variables */
}

.my-custom-theme.dark {
  --background: 240 10% 4%;
  --foreground: 0 0% 100%;
  --primary: 200 80% 60%;
  /* ... define all required variables */
}
```

### 2. Update Theme Type

Add your theme to `src/lib/theme.ts`:

```typescript
export type Theme =
  | "light"
  | "dark"
  | "system"
  | "amethyst-haze"
  | "solar-dusk"
  | "nature"
  | "my-custom-theme"; // Add here
```

### 3. Update Theme Utilities

Update the `getStoredTheme()` and `applyTheme()` functions in `src/lib/theme.ts` to include your theme.

### 4. Add to Settings

Update `src/components/SettingsView.tsx` to include your theme in the themes array:

```typescript
const themes = [
  // ... existing themes
  { 
    id: "my-custom-theme", 
    name: "My Custom Theme", 
    description: "A custom theme I created" 
  },
];
```

---

## Theme Sources

All themes (except Light and Dark) are sourced from [tweakcn](https://tweakcn.com), a community-driven collection of shadcn/ui themes.

- **Amethyst Haze**: [tweakcn.com/r/themes/amethyst-haze](https://tweakcn.com/r/themes/amethyst-haze.json)
- **Solar Dusk**: [tweakcn.com/r/themes/solar-dusk](https://tweakcn.com/r/themes/solar-dusk.json)
- **Nature**: [tweakcn.com/r/themes/nature](https://tweakcn.com/r/themes/nature.json)

All themes have been converted to HSL format for consistency and compatibility.

---

## Accessibility

All themes are designed with accessibility in mind:

- **Contrast Ratios**: Meet WCAG AA standards
- **Color Blindness**: Tested for common color vision deficiencies
- **Readability**: High contrast between text and backgrounds
- **UI Elements**: Clear distinction between interactive elements

---

## Theme Comparison

| Theme | Mood | Primary Color | Best Use Case |
|-------|------|---------------|---------------|
| Light | Professional | Blue | General work, daytime |
| Dark | Focused | Blue | Night work, low-light |
| Amethyst Haze | Elegant | Purple | Creative, extended use |
| Solar Dusk | Warm | Orange | Comfortable atmosphere |
| Nature | Calm | Green | Relaxing, nature-inspired |

---

## Tips

1. **Try them all**: Each theme offers a unique experience
2. **Match your environment**: Use dark themes in dark rooms
3. **Consider your task**: Different themes work better for different tasks
4. **Personal preference**: Choose what feels most comfortable to you
5. **Change anytime**: Themes can be switched instantly without restarting

---

**Last Updated**: January 2024  
**Version**: 0.1.0