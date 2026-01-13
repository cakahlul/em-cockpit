# 🚀 The EM Cockpit

A **Command Center for Engineering Managers** — Manage tickets, PRs, incidents, and specs from a single spotlight-style interface.

![EM Cockpit Banner](https://via.placeholder.com/800x200?text=The+EM+Cockpit)

## ✨ Features

- **Flight Console**: Global hotkey (`Alt+Space`) spotlight search for everything.
- **PR Radar**: Visual dashboard for Pull Request monitoring, stale checking, and review prioritization.
- **Incident Bridge**: Real-time incident tracking and tray status indicators.
- **Privacy First**: Secure credential storage and local-first data architecture.
- **Glassmorphism UI**: Beautiful, modern, native-feeling interface.

## 🛠️ Prerequisites

Before you begin, ensure you have the following installed:

1.  **Rust**: [Install Rust](https://www.rust-lang.org/tools/install) (latest stable).
2.  **Node.js**: [Install Node.js](https://nodejs.org/) (v16 or newer recommended).
3.  **System Dependencies** (Linux only): `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`.

## 🏃‍♂️ Getting Started

### 1. Install Dependencies

Install the frontend dependencies:

```bash
npm install
```

### 2. Run in Development Mode

Run the app locally with hot-reloading for both Rust and Vue:

```bash
npm run tauri dev
```

*The first run might take a few minutes as it compiles the Rust dependencies.*

### 3. Run Tests

**Backend (Rust)**:
```bash
cd src-tauri
cargo test
```

**Integration Tests**:
```bash
cd src-tauri
cargo test --test integration_tests
```

## 📦 Building for Production

To create an optimized release build:

```bash
npm run tauri build
```

The output application will be located in `src-tauri/target/release/bundle`.

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Alt + Space` | Toggle Flight Console (Search) |
| `Esc` | Close Flight Console |

## 🧩 Architecture

- **Frontend**: Vue 3, TypeScript, Vite, CSS Variables (Glassmorphism).
- **Backend**: Rust (Tauri), SQLite, Keyring.
- **Communication**: Tauri Events & Commands.

## 📄 License

Proprietary / Internal Use Only.
