# Development Guide

This guide covers everything you need to develop and contribute to Videre.

## Prerequisites

- [Bun](https://bun.sh/) (or npm/yarn/pnpm)
- [Rust](https://rustup.rs/) (for Tauri)
- [Docker](https://www.docker.com/) (for test database)

## Quick Start

```bash
# Install dependencies
bun install

# Start the test PostgreSQL database
docker-compose up -d

# Run in development mode (web only)
bun run dev

# Run as Tauri desktop app
bun run tauri dev
```

## Test Database

The project includes a Docker Compose configuration with a pre-populated PostgreSQL database for development and testing.

### Connection Details

| Field    | Value        |
|----------|--------------|
| Host     | `localhost`  |
| Port     | `5432`       |
| Database | `videre_test`|
| Username | `videre`     |
| Password | `videre`     |

### Sample Data - Greek Mythology Theme

The test database includes a complete Greek mythology dataset across **7 tables**:

- **Primordials** (7) - The first beings: Chaos, Gaia, Uranus, Nyx, Erebus, Tartarus, Eros
- **Titans** (12) - Second generation: Cronus, Rhea, Oceanus, Hyperion, Themis, Mnemosyne, and more
- **Gods** (36) - Olympians and their descendants
- **Heroes** (10) - Legendary heroes: Heracles, Perseus, Achilles, Odysseus, Theseus, Jason, etc.
- **Creatures** (13) - Monsters and beasts: Medusa, Minotaur, Hydra, Chimera, Cerberus, Sphinx, etc.
- **Quests** (12) - Epic adventures and labors
- **Artifacts** (17) - Legendary items: Zeus' Lightning Bolt, Scythe of Cronus, etc.

**Plus 4 views:**
- `mythology_summary` - Count of all entities by category
- `hero_achievements` - Hero stats with quests completed and monsters slain
- `divine_lineage` - Complete family tree from Primordials → Titans → Gods
- `artifact_registry` - Artifact ownership and forging details

### Managing the Database

```bash
# Start the database
docker-compose up -d

# Stop (keeps data)
docker-compose stop

# Stop and remove containers (keeps data in volume)
docker-compose down

# Stop and remove everything including data
docker-compose down -v

# View logs
docker-compose logs -f postgres
```

## Project Structure

```
├── src/                  # React frontend
│   ├── components/       # UI components
│   ├── stores/           # Zustand state management
│   ├── themes/           # Theme definitions
│   └── lib/              # Utilities
├── src-tauri/            # Rust backend
│   └── src/
│       └── lib.rs        # Tauri commands & PostgreSQL integration
├── public/               # Static assets
└── docker-compose.yml    # Test database configuration
```

## Tech Stack

### Frontend
- **React 19** - UI framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Utility-first styling
- **Zustand** - State management
- **Vite** - Build tool

### Backend
- **Tauri 2** - Desktop app framework
- **Rust** - Backend language
- **tokio-postgres** - PostgreSQL client

## Commands

```bash
# Development
bun run dev          # Start Vite dev server (web only)
bun run tauri dev    # Start Tauri app in dev mode

# Building
bun run build        # Build frontend
bun run tauri build  # Build production app

# Preview
bun run preview      # Preview production build
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `⌘/Ctrl + T` | Create new empty tab |
| `⌘/Ctrl + W` | Close active tab |
| `⌘/Ctrl + 1-9` | Switch to tab by number |

## Theming

Themes are defined in `src/themes/` using OKLCH color space with shadcn-style CSS variables.

### Available Themes
- **Amethyst Haze** - Purple-tinted elegance
- **Solar Dusk** - Warm sunset tones
- **Nature** - Fresh green palette

Each theme includes light and dark variants.

### Theme Variables
All colors use HSL values as CSS variables:
- `--background`, `--foreground`
- `--primary`, `--secondary`, `--accent`
- `--muted`, `--card`, `--border`

## Architecture Decisions

### Minimal Dependencies
We intentionally keep dependencies minimal to reduce supply chain attack surface. Before adding a new dependency, consider:
1. Is it absolutely necessary?
2. How well maintained is it?
3. How many transitive dependencies does it bring?

### State Management
We use Zustand for all state management, including the tab system. State is organized in `src/stores/`.

### No Router
The app uses a tab-based navigation system instead of a traditional router. This keeps the mental model simple and avoids unnecessary dependencies.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Guidelines

- Follow existing code style
- Keep dependencies minimal
- Test with the provided Docker database
- Update documentation as needed

## Troubleshooting

### Database connection fails
1. Ensure Docker is running: `docker ps`
2. Check if the container is up: `docker-compose ps`
3. Try restarting: `docker-compose down && docker-compose up -d`

### Tauri build fails
1. Ensure Rust is installed: `rustc --version`
2. Update Rust: `rustup update`
3. Check Tauri prerequisites: https://tauri.app/v1/guides/getting-started/prerequisites

### Port already in use
If port 5432 is in use, modify `docker-compose.yml` to use a different port:
```yaml
ports:
  - "5433:5432"  # Use 5433 instead
```
Then connect using port 5433 in the app.