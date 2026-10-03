# CryptForge

[![Rust](https://img.shields.io/badge/Rust-dea584?style=flat-square&logo=rust)](#) [![TypeScript](https://img.shields.io/badge/TypeScript-3178c6?style=flat-square&logo=typescript)](#) [![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#)

> A classic roguelike dungeon crawler — procedural dungeons, permadeath, native desktop speed — running on Tauri 2 and Rust.

CryptForge is a turn-based roguelike that runs as a native desktop application. The game logic lives in Rust for correctness and speed; the UI is rendered in React with keyboard controls for core gameplay. Runs feature procedurally generated dungeons, enemy AI, inventory management, and permadeath.

## Features

- **Procedural dungeon generation** — seeded layouts, enemies, and loot
- **Turn-based tactical combat** — plan each move; no twitch reactions required
- **Keyboard-first controls** — movement, combat, and inventory shortcuts; class/modifier and crafting selections require a mouse
- **Native desktop performance** — Rust game logic, React UI in a Tauri webview
- **Permadeath** — every decision matters; death ends the run and clears its save

## Quick Start

### Prerequisites

- Node.js 22.12+ or 24–25 and npm 10+ (package engines allow 20–25; locked Vite/jsdom need at least 20.19 on Node 20)
- Rust stable toolchain (`rustup`)
- Tauri system dependencies: [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/)

### Installation

```bash
git clone https://github.com/saagpatel/CryptForge
cd CryptForge
npm ci
```

### Usage

```bash
# Start in development mode
npm run tauri dev

# Lean dev mode (lower disk usage)
npm run dev:lean
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop shell | Tauri 2 |
| Frontend | React 19, TypeScript, Vite |
| Game logic | Rust |
| Testing | Vitest, Testing Library |

## Architecture

Game state and logic are owned by the Rust backend, exposed to the React frontend via Tauri commands. The frontend handles rendering and input, while Rust enforces all game rules — turn resolution, dungeon generation, and entity state — using seeded randomness. Loading a save reseeds the RNG from the seed plus turn rather than restoring its prior state.

## License

MIT

See [CONTRIBUTING.md](CONTRIBUTING.md#verification) for focused checks, the full verification lanes, and local preview limits.
