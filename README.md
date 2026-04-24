<h1 align="center">Videre</h1>

<p align="center">
  A simplicty and privacy focused PostgreSQL inspector.
</p>

<p align="center">
  <img src="magnifying-glass.png" alt="Videre" width="600" />
</p>


## Why Videre?

Sometimes you just need to quickly connect to a database and browse your data. Videre is designed for exactly that — a lightweight tool for when you want to inspect tables without any setup overhead.

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

Videre is currently **read-only** — perfect for browsing and understanding your data without risk of accidental changes.

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

**More to inspect**
- [ ] Sequences
- [ ] Constraints
- [ ] Stored procedures / Functions

**Usability**
- [ ] Table filtering and search
- [x] Column sorting
- [ ] Export to CSV/JSON
- [ ] Connection profiles (save multiple databases)

**Edit capabilities**
- [ ] Insert rows
- [ ] Update rows
- [ ] Delete rows
- [ ] Query editor

**More databases**
- [ ] MySQL support
- [ ] SQLite support

## License

[MIT](./LICENSE)

---

<p align="center">
  <sub>Built for developers who just want to see their data.</sub>
</p>
