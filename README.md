<p align="center">
  <h1 align="center">Videre</h1>
  <p align="center">
    A simple and private PostgreSQL inspector.
  </p>
</p>

---

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

## Installation

### Download

> Coming soon — Pre-built binaries for macOS, Windows, and Linux.

### Build from Source

Requires [Bun](https://bun.sh/) (or npm) and [Rust](https://rustup.rs/).

```bash
# Clone the repository
git clone https://github.com/yourusername/videre.git
cd videre

# Install dependencies
bun install

# Build the app
bun run tauri build
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
bun install
docker-compose up -d    # Start test database
bun run tauri dev       # Run the app
```

## Tech Stack

| Layer    | Technology |
|----------|------------|
| Frontend | React, TypeScript, Tailwind CSS |
| Backend  | Rust, Tauri 2 |
| Database | tokio-postgres |
| State    | Zustand |

## Roadmap

- [ ] MySQL support
- [ ] SQLite support
- [ ] Table filtering and search
- [ ] Column sorting
- [ ] Export to CSV/JSON
- [ ] Connection profiles (save multiple databases)

## License

[MIT](./LICENSE)

---

<p align="center">
  <sub>Built for developers who just want to see their data.</sub>
</p>
