<h1 align="center">Videre</h1>

<p align="center">
  A beautiful, simple, and security-focused PostgreSQL reader.
</p>

<p align="center">
  <img src="magnifying-glass.png" alt="Videre" width="600" />
</p>


## Why Videre?

Sometimes you just need to quickly connect to a database and understand its data. Videre is designed for exactly that — a lightweight tool for inspecting PostgreSQL without the clutter and complexity of a traditional database client.

The initial and primary goal is to make the best possible read-only experience. Editing will come later as an explicit, opt-in capability without replacing the simplicity of the reader.

### Product Direction

#### Phase 1 — Reader Mode

Reader Mode is the current and primary focus. It provides a beautiful, simple, and secure environment for browsing data, understanding database structure, and following relationships without exposing mutation controls.

Read-only is a first-class product mode, not a temporary limitation. It will remain the default even after editing is introduced.

#### Phase 2 — Edit Mode

Once Reader Mode is "finished", Videre will add an explicit Edit Mode for inserting, updating, and deleting data. Editing will be clearly indicated and deliberately enabled, while Reader Mode remains available for users who only want to inspect a database.

Videre remains a database inspector in both modes: Reader Mode is for understanding, and Edit Mode adds controlled mutation capabilities.

### Simple

Connect to your database, browse your tables. That's the core experience. We focus on doing this one thing well.

- **Tab-based interface** — Open multiple tables side by side
- **Keyboard navigation** — `⌘/Ctrl + T`, `⌘/Ctrl + W`, `⌘/Ctrl + [1-9]`
- **Pagination** — Navigate through large tables
- **Themes** — Light and dark mode with multiple color themes

### Private & Secure

Your data is yours. Videre runs entirely on your machine.

- **Zero telemetry** — No tracking, no analytics, no data collection
- **Local only** — Your credentials and data never leave your machine
- **Minimal dependencies** — We keep the dependency count low to reduce supply chain risks
- **Open source** — The code is fully auditable

## What You Can Inspect

Videre currently operates in **Reader Mode** — a read-only experience for browsing and understanding your data without exposing accidental mutation controls.

- **Tables** — Browse data with pagination
- **Indexes** — View index configurations
- **Roles** — See database roles and permissions
- **Foreign keys** — Understand table relationships
- **Views** — Inspect view data

---

**Can you vibe code this?** Yeah, probably! But I already spent the time and tokens so you don't have to :)

---

## Installation

### Download

> Coming soon — Pre-built binaries for macOS, Windows, and Linux.

### Build from Source

Requires [Rust](https://rustup.rs/), [Trunk](https://trunkrs.dev/), and the [Tauri CLI](https://tauri.app/).

```bash
# Install build tools
rustup target add wasm32-unknown-unknown
cargo install trunk
cargo install tauri-cli

# Clone the repository
git clone https://github.com/yourusername/videre.git
cd videre

# Build the app
cargo tauri build
```

The built application will be in `src-tauri/target/release/`.

## Development

See [DEVELOPMENT.md](./DEVELOPMENT.md) for detailed setup instructions, including:
- Running the dev environment
- Test database with sample data
- Project structure
- Contributing guidelines

**Quick start:**

```bash
docker-compose up -d    # Start test database
cargo tauri dev         # Run the app
```

## Tech Stack

| Layer    | Technology |
|----------|------------|
| Frontend | Leptos 0.7 (Rust → WASM), Trunk |
| Backend  | Rust, Tauri 2 |
| Database | tokio-postgres |
| Styling  | Hand-written CSS (OKLCH) |

## Roadmap

**Phase 1 — Reader Mode**
- [ ] Trustworthy server-side sorting, filtering, and pagination
- [ ] Copy cells and rows, inspect full values, and export CSV/JSON
- [ ] Inspect columns, types, constraints, indexes, and relationships
- [ ] Improve schema navigation, object search, and connection context
- [ ] Add focused read-only diagnostics for activity, locks, and database health

**Phase 2 — Edit Mode**
- [ ] Explicit Reader/Edit mode selection
- [ ] Insert rows
- [ ] Update rows
- [ ] Delete rows
- [ ] Clear previews and confirmations for mutations
- [ ] Focused query workspace

**Future database support**
- [ ] Evaluate MySQL and SQLite after the PostgreSQL experience is excellent

## License

[MIT](./LICENSE)

---

<p align="center">
  <sub>Built for developers who just want to see their data.</sub>
</p>
