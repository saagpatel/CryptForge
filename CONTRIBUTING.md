# Contributing

Thank you for your interest in contributing!

## Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/your-feature`
3. Make your changes
4. Commit using [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `chore:`, etc.
5. Push and open a pull request

## Reporting Issues

Open a [GitHub Issue](../../issues) with a clear description and steps to reproduce.

## Code Style

Follow the existing conventions in the codebase.

## Verification

Run commands from the repository root. Use Node 22.12+ or 24–25 (within the package engine range), npm 10+, stable Rust
with rustfmt and Clippy, and the native Tauri prerequisites for your OS.
`package-lock.json` is the CI install authority: `npm ci`. The prepare hook
configures Husky; use a standalone clone if another worktree must retain its Git
configuration. For a hook-free inspection install, `npm ci --ignore-scripts`
skips lifecycle scripts; this is not a substitute for CI's normal installation.

For a focused frontend change:

```sh
npm test -- src/lib/api.test.ts
```

For a focused Rust change, use the existing safe-cwd wrapper and a test-name
filter after the manifest argument:

```sh
node scripts/run-in-safe-cwd.mjs -- cargo test --manifest-path src-tauri/Cargo.toml dungeon
```

For broader verification, the maintained scripts are:

```sh
npm run verify:preflight
npm run verify:frontend
npm run verify:rust
# Or run all three lanes:
npm run verify
```

`verify:frontend` runs TypeScript/build and all Vitest tests. `verify:rust` runs
Rust tests, rustfmt in check mode, and Clippy with warnings denied. Root Makefile
targets cover Rust only. CI adds performance capture/budgets in
[ci-pr.yml](.github/workflows/ci-pr.yml); the
[managed verify list](.codex/verify.commands) currently covers guards and
performance, so it does not replace the correctness commands above.

For UI/input changes, also inspect the changed flow with synthetic game state.
`npm run dev` is a browser preview; native IPC needs `npm run tauri -- dev`.
Native launch may write application state. Use disposable game data and avoid
cleanup/lean-dev scripts on a preserved checkout. Browser appearance does not
prove Rust game rules or a signed desktop release. Documentation-only changes
do not require launching the game or collecting performance baselines.
