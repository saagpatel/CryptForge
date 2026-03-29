# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2026-03-24

### Added
- Project scaffolding: Tauri 2 + React 19 + Vite
- Core Rust types: entities, map, IPC types, and all game data structures
- Dungeon generation: BSP, cellular automata, room types, and floor generation
- Entity definitions and spawning: full bestiary, items, and placement
- FOV shadowcasting and pathfinding
- Turn system and combat
- Enemy AI behaviors
- Status effects, inventory management, and leveling system
- IPC commands, SQLite persistence, and Ollama flavor engine
- Frontend types, API, renderer, UI components, input, and menus
- Audio engine, tileset system, and animations
- Comprehensive test suite: 121 tests across all modules
- Secret rooms: hidden passages revealed by bumping walls
- Phase 2 features: mouse, shake, sprites, biomes, auto-explore, ranged combat, targeting, shops, interactables, achievements, and seed sharing
- Phase 3 feature expansion: 16 features across 4 sub-phases
- Complete CI/CD, release automation, and platform testing infrastructure
- Lean dev workflow and cleanup scripts
- Verification doctor workflow and session audit artifacts

### Fixed
- Regenerate Windows app icon
- Invoke npm through cmd on Windows
- Resolve npm on Windows preflight
- Bootstrap CI baselines and sync backend snapshot
- Set Rust toolchain explicitly and align workflows with npm
- Fix 25 bugs from comprehensive code audit
- Fix 14 bugs and issues from comprehensive codebase audit
- Fix 4 bugs: equipment speed, enemy on-hit effects, arena cycle, and toast leak
- Fix 9 bugs from comprehensive codebase audit

### Changed
- Sync frontend tests and npm verification assets
- Format Rust source
- Add local Tauri CLI dependency
- Remove unused tokio dependency, shell plugin, and icon variants
- Slim docs and add lightweight CI
- Remove non-runtime bloat and generated artifacts
- Finalize Codex OS bootstrap baseline and guardrails
- Add definitive implementation plan
