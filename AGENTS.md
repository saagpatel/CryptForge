<!-- comm-contract:start -->

## Communication Contract

- Inherit global Codex communication and reporting rules from `/Users/d/.codex/AGENTS.override.md` and `/Users/d/.codex/policies/communication/BigPictureReportingV1.md`.
- Repo-specific instructions below add project constraints only; do not restate global voice or status-reporting rules here.
<!-- comm-contract:end -->

## Inherited Operating Rules

- Inherit global git, review/fix, testing, docs, skill-use, and reporting gates from `/Users/d/.codex/AGENTS.md` and active session instructions.
- Use `.codex/verify.commands` and `.codex/scripts/run_verify_commands.sh` as this repo-local verification authority when present.
- Keep the project-specific portfolio constraints below as the source of truth for runtime, privacy, and release risks.

<!-- portfolio-context:start -->

# Portfolio Context

## What This Project Is

CryptForge is a native desktop roguelike built with Tauri 2, Rust game logic, and a React UI. It focuses on procedural dungeon generation, turn-based tactical combat, keyboard-first play, inventory management, enemy AI, and permadeath.

## Current State

The repo is active game/product work. Existing local changes include PR-template metadata and an untracked lockfile, so this recovery pass should only add portfolio context.

## Stack

| Layer         | Technology                 |
| ------------- | -------------------------- |
| Desktop shell | Tauri 2                    |
| Frontend      | React 19, TypeScript, Vite |
| Game logic    | Rust                       |
| Testing       | Vitest, Testing Library    |

## How To Run

```bash
# Start in development mode
npm run tauri dev

# Lean dev mode (lower disk usage)
npm run dev:lean
```

## Known Risks

- The on-disk folder name still carries whitespace drift that can make scripted paths easier to misread.
- Rust owns the game simulation; keep rules, dungeon generation, turn resolution, and entity state deterministic.
- Preserve keyboard-first controls and avoid UI-only rule enforcement.
- Keep Tauri/Rust dependency updates separate from gameplay behavior changes.

## Next Recommended Move

Resolve the folder/path and PR-template drift separately, then verify the Rust game rules and React input flow before shipping gameplay changes.

<!-- portfolio-context:end -->
