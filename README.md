# CryptForge

[![Rust](https://img.shields.io/badge/Rust-dea584?style=flat-square&logo=rust)](#) [![TypeScript](https://img.shields.io/badge/TypeScript-3178c6?style=flat-square&logo=typescript)](#) [![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#)

> A classic roguelike dungeon crawler — procedural dungeons, permadeath, native desktop speed — running on Tauri 2 and Rust.

CryptForge is a turn-based roguelike that runs as a native desktop application. The game logic lives in Rust for correctness and speed; the UI is rendered in React with full keyboard navigation. Procedurally generated dungeons, enemy AI, inventory management, and permadeath make every run distinct.

## Features

- **Procedural dungeon generation** — new layouts, enemies, and loot every run
- **Turn-based tactical combat** — plan each move; no twitch reactions required
- **Keyboard-first controls** — full game playable without touching a mouse
- **Native desktop performance** — Rust game logic, no browser overhead
- **Permadeath** — every decision matters; death starts a fresh run

## Quick Start

### Prerequisites

- Node.js 18+
- Rust stable toolchain (`rustup`)
- Tauri system dependencies: [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/)

### Installation

```bash
git clone https://github.com/saagpatel/CryptForge
cd CryptForge
npm install
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

Game state and logic are owned by the Rust backend, exposed to the React frontend via Tauri commands. The frontend handles rendering and input, while Rust enforces all game rules — turn resolution, dungeon generation, and entity state — keeping the simulation deterministic and fast.

## License

MIT
