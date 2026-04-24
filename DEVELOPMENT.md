# Development Guide

This guide covers everything you need to develop and contribute to Videre.

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain + `wasm32-unknown-unknown` target)
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)
- [Tauri CLI](https://tauri.app/) (`cargo install tauri-cli`)
- [Docker](https://www.docker.com/) (for test database)

### Rust WASM Target

```bash
rustup target add wasm32-unknown-unknown
```

## Quick Start

```bash
# Start the test PostgreSQL database
docker-compose up -d

# Run as Tauri desktop app (this also starts Trunk)
cargo tauri dev
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
- **Artifacts** (17) - Legendary items: Zeus Lightning Bolt, Scythe of Cronus, etc.

**Plus 4 views:**
- `mythology_summary` - Count of all entities by category
- `hero_achievements` - Hero stats with quests completed and monsters slain
- `divine_lineage` - Complete family tree from Primordials to Titans to Gods
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
Daedalus/
  index.html                # Trunk entry point
  style.css                 # Global styles (OKLCH CSS variables)
  Trunk.toml                # Trunk build configuration
  docker-compose.yml        # Test database configuration
  src-leptos/               # Leptos frontend (compiles to WASM)
    Cargo.toml
    src/
      main.rs               # Entry point, provides global context
      tauri.rs              # Tauri IPC invoke bridge
      theme.rs              # Theme/mode management (localStorage + CSS)
      types.rs              # Shared IPC data types (mirrors backend)
      components/           # Reusable UI components
        data_table.rs       # Generic sortable data table
        drawer.rs           # Slide-out side panel
        empty.rs            # Empty state placeholders
        icons.rs            # Shared SVG icon functions
        shell.rs            # Main app layout (sidebar + tabs + content)
        sidebar.rs          # Left navigation sidebar
        tab_bar.rs          # Horizontal tab strip
      pages/                # Full-screen views rendered in tabs
        connection.rs       # Database connection form
        indexes.rs          # Indexes overview
        roles.rs            # Roles and permissions
        settings.rs         # Theme picker and about info
        table.rs            # Paginated table data viewer
      stores/               # Reactive state (Leptos signals)
        connection_store.rs # Saved connections (persisted via Tauri store)
        db_store.rs         # Database metadata (tables, FKs, indexes, roles)
        tab_store.rs        # Tab management (open, close, switch)
  src-tauri/                # Tauri backend (native Rust)
    Cargo.toml
    tauri.conf.json
    src/
      main.rs               # Tauri bootstrap
      lib.rs                # Plugin + command registration
      db.rs                 # PostgreSQL queries and IPC types
```

## Tech Stack

### Frontend
- **Leptos 0.7** - Reactive Rust UI framework (CSR mode)
- **Trunk** - WASM build tool and dev server
- **CSS** - Hand-written styles using OKLCH color space

### Backend
- **Tauri 2** - Desktop app framework
- **Rust** - Backend language
- **tokio-postgres** - Async PostgreSQL client

### IPC
The frontend communicates with the backend via Tauri's `invoke` mechanism (`window.__TAURI__.core.invoke`). Data is serialized as JSON via `serde`. Both sides define matching types independently (see `src-leptos/src/types.rs` and `src-tauri/src/db.rs`).

## Commands

```bash
# Development
cargo tauri dev              # Start Tauri app with hot-reload (runs Trunk automatically)

# Building
cargo tauri build            # Build production desktop app

# Frontend only (without Tauri shell)
cd src-leptos && trunk serve # Serve WASM frontend at localhost:1420
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + T` | Create new empty tab |
| `Cmd/Ctrl + W` | Close active tab |
| `Cmd/Ctrl + 1-9` | Switch to tab by number |

## Theming

Themes are defined in `style.css` using OKLCH color space with CSS custom properties.

### Available Themes
- **Amethyst Haze** - Purple-tinted elegance
- **Solar Dusk** - Warm sunset tones
- **Nature** - Fresh green palette

Each theme includes light and dark variants. Theme and mode preferences are persisted in `localStorage`.

## Architecture Decisions

### Minimal Dependencies
We intentionally keep dependencies minimal to reduce supply chain attack surface. Before adding a new dependency, discuss it first and consider:
1. Is it absolutely necessary?
2. How well maintained is it?
3. How many transitive dependencies does it bring?

### State Management
All state is managed via Leptos reactive signals, provided through context. Stores are organized in `src-leptos/src/stores/`.

### No Router
The app uses a tab-based navigation system instead of a traditional router. This keeps the mental model simple and avoids unnecessary dependencies.

### Type Duplication
The frontend (`types.rs`) and backend (`db.rs`) each define their own copies of the IPC data types. This is intentional. The two crates compile for different targets (WASM vs native) with different `Cargo.lock` files, and keeping them independent avoids workspace coupling for a small number of simple structs.

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
3. Ensure WASM target is installed: `rustup target add wasm32-unknown-unknown`
4. Ensure Trunk is installed: `cargo install trunk`
5. Check Tauri prerequisites: https://tauri.app/start/prerequisites/

### Port already in use
If port 5432 is in use, modify `docker-compose.yml` to use a different port:
```yaml
ports:
  - "5433:5432"  # Use 5433 instead
```
Then connect using port 5433 in the app.
