<p align="center">
  <h1 align="center">Videre</h1>
  <p align="center">
    <strong>See your database. Nothing else.</strong>
  </p>
  <p align="center">
    A simple, private, and fast PostgreSQL inspector.
  </p>
</p>

---

## Why Videre?

Most database tools try to do everything. They're packed with features you'll never use, drowning in menus, and take 10 minutes just to figure out how to connect.

**Videre does one thing well: let you see your data.**

### 🎯 Simple

Connect to your database, browse your tables. Your data, clearly presented.

### 🔒 Private

- **Zero telemetry** — We don't track anything. Ever.
- **No analytics** — No usage stats, no "anonymous" data collection.
- **Local only** — Your credentials and data never leave your machine.

### 🛡️ Secure

- **Minimal dependencies** — Fewer deps = smaller attack surface. We're serious about supply chain security.
- **Open source** — Every line of code is auditable. Verify our claims yourself.
- **Native app** — Built with Tauri (Rust), not Electron. No bundled Chromium, no Node.js runtime.

### ⚡ Fast

Rust backend. Native performance. Starts in milliseconds, not seconds.

## Features

- **Tab-based interface** — Open multiple tables side by side
- **Keyboard-first** — Navigate with `⌘/Ctrl + T`, `⌘/Ctrl + W`, `⌘/Ctrl + [1-9]`
- **Pagination** — Handle large tables gracefully
- **Themes** — Light/dark mode with multiple color themes

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

**Runtime dependencies:** ~10 packages. That's it.

## Roadmap

- [ ] MySQL support
- [ ] SQLite support
- [ ] Table filtering and search
- [ ] Column sorting
- [ ] Export to CSV/JSON
- [ ] Connection profiles (save multiple databases)

## Philosophy

> "Perfection is achieved, not when there is nothing more to add, but when there is nothing left to take away." — Antoine de Saint-Exupéry

We believe the best tools are the ones that get out of your way. Videre will always prioritize simplicity over features.

## License

[MIT](./LICENSE)

---

<p align="center">
  <sub>Built for developers who just want to see their data.</sub>
</p>
