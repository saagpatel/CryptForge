# CryptForge — Portfolio Disposition

**Status:** Active — working Tauri 2 + Rust + React roguelike on
`origin/main`, no release-readiness doc yet. Disposition is **not**
Release Frozen; the gate is "decide whether to package this for
distribution."

> Also worth knowing: this repo lives at a filesystem path with a
> **leading space** in the directory name. See "Filesystem oddity"
> below.

---

## Filesystem oddity

On disk, the canonical local checkout is:

```
/Users/d/Projects/Fun:GamePrjs/ CryptForge/
                                ^-- leading space, not a typo
```

The leading space is preserved across operations and is the actual
directory name. Common pitfalls:

- `cd /Users/d/Projects/Fun:GamePrjs/CryptForge` (no space) **will
  fail** — wrong path.
- Unquoted shell arguments referring to this path break:
  `find /Users/d/Projects -name CryptForge` returns the dir, but
  `cd $REPO` where `REPO` is the unquoted result will treat the
  space as an argument separator.
- Tab completion in most shells handles it cleanly if you start
  typing ` Cry` (with the leading space). Backtick or `$(...)`
  composition needs explicit quoting.

**Operator guidance:** when writing scripts or tooling that
references this path, always use double quotes:
`cd "/Users/d/Projects/Fun:GamePrjs/ CryptForge"`.

This is a different shape from the OrbitForge typo-dup (where the
typo'd path was the _stale_ sibling). Here the leading space is on
the canonical path and there is no clean-named sibling — so it
can't be "fixed" by deleting a dup. The right move is to either
live with it or rename the directory (operator-side, requires
updating any tooling references).

---

## Verification posture

This repo has **no `legacy-origin` remote** — clean migration state.

Specifically verified on `origin/main` (default branch is `main`):

- Tip: `2dc490c` chore: add pull request template
- Substantive feature commits on `origin/main`:
  - `5cab6ba` Merge PR #5: codex/fix/windows-npm-cmd
  - `b6caac1` fix(ci): invoke npm through cmd on windows
  - `32c27a7` fix(ci): resolve npm on windows preflight
  - `6cfd718` fix(perf): bootstrap CryptForge CI baselines
  - `1036842` fix(rust): sync CryptForge backend snapshot
- Source tree: standard Tauri 2 + Rust + React layout
- No `docs/` directory before this file
- Recent commits emphasize cross-platform CI hardening (Windows fixes)

---

## Current state in one paragraph

CryptForge is a turn-based roguelike dungeon crawler that runs as a
native desktop application. Game logic in Rust for correctness and
speed; UI in React with full keyboard navigation. Procedurally
generated dungeons, enemy AI, inventory management, permadeath —
every run distinct. Tauri 2 desktop shell. The recent commit pattern
on `origin/main` is mostly CI hardening (Windows toolchain, npm
preflight, Rust toolchain explicit, perf baselines bootstrapped) —
the operator was prepping for cross-platform distribution but
hasn't written the release-readiness doc yet.

For full detail see `README.md`.

---

## Why "Active" instead of Release Frozen

Same shape as Conductor / SnippetLibrary / Chronomap /
ScreenshotAnnotate. The signing cluster (10 repos) all have
release-readiness docs. CryptForge doesn't. The next move is
operator decision-time about distribution, not signing-credential-time.

CryptForge is **distinct from the macOS-only members** of the
signing cluster because the recent CI work explicitly targets
Windows. Distribution decision needs to address:

- macOS .app via Apple signing + notarization
- Windows installer via code-signing certificate
- Linux AppImage (no signing required)
- Cross-platform GitHub Releases vs per-platform store submissions

---

## Possible next moves (operator choice)

### Option 1 — Package for cross-platform distribution

Required scope:

1. Write `docs/RELEASE-READINESS.md` covering all three platforms
2. Wire signing for both macOS (Apple Dev ID) and Windows (code cert)
3. Build cross-platform release pipeline (likely GitHub Actions
   matrix per the existing Windows CI fixes pattern)
4. Cut v0.1.0 release with installers for all three platforms

Estimated effort: ~6 hours (more than other Tauri repos because of
cross-platform scope; the Windows CI already exists, just needs
signing wiring)

### Option 2 — Itch.io distribution

Roguelikes have an established audience on itch.io. Skip code-signing
entirely; itch.io accepts unsigned builds and handles the discovery.
Build via `pnpm tauri build` per platform, upload via butler CLI.

Estimated effort: ~2 hours. Trade-off: lose platform installer trust
signals but gain a viable distribution channel without credential
overhead.

### Option 3 — Open-source as a build-locally game

Document the local-build path well, no signing or distribution. Users
who want it clone and run.

Estimated effort: ~30 minutes.

---

## Recommendation (informational)

**Option 2 (itch.io) is probably the right call** for CryptForge
specifically:

- Roguelikes have a thriving itch.io community
- Credential overhead is the biggest sunk cost of Option 1 — itch.io
  bypasses it entirely
- Cross-platform delivery is solved (itch.io handles per-OS
  downloads natively)
- The "permadeath turn-based roguelike" framing is _exactly_ itch.io's
  audience taste profile

Option 1 is correct if the operator's portfolio strategy is
"everything we ship lives on saagpatel.com / GitHub Releases"
consistency. Otherwise itch.io is a better fit per-repo.

But this is operator-judgment territory and depends on the broader
distribution-channel strategy across the portfolio.

---

## Portfolio operating system instructions

| Aspect                                 | Posture                                                                                                                                                                |
| -------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Portfolio status                       | `Active`                                                                                                                                                               |
| Filesystem oddity                      | Leading-space in `/Users/d/Projects/Fun:GamePrjs/ CryptForge/` path — tooling must quote it                                                                            |
| Next packet shape                      | "Decide between Option 1 / 2 / 3"                                                                                                                                      |
| Review cadence                         | Resume normal cadence — this row needs decision-time                                                                                                                   |
| Resurface conditions                   | Once the operator picks an option, surface a packet for the work each option implies                                                                                   |
| Do **not** auto-add to signing cluster | The cluster is for repos that already have release-readiness docs. CryptForge doesn't yet. Plus cross-platform scope is different from the macOS-only cluster members. |

---

## Reactivation procedure (for the next code session)

1. Quote the path: `cd "/Users/d/Projects/Fun:GamePrjs/ CryptForge"`
2. Verify `git branch -vv` shows `main` tracking `origin/main`.
   This repo has no `legacy-origin` so the trap isn't here.
3. Delete stale `codex/*` branches that pre-date the Windows CI
   fix commits.
4. Re-run `pnpm install && pnpm tauri build` to confirm the
   toolchain still works after the freeze (across all target
   platforms if testing cross-platform).
5. If picking Option 1 or 2, write the release-readiness doc first.

---

## Last known reference

| Field                    | Value                                                          |
| ------------------------ | -------------------------------------------------------------- |
| Canonical local path     | `/Users/d/Projects/Fun:GamePrjs/ CryptForge/` (leading space!) |
| `origin/main` tip        | `2dc490c` chore: add pull request template                     |
| Last substantive commit  | `5cab6ba` Merge PR #5: codex/fix/windows-npm-cmd               |
| Default branch           | `main`                                                         |
| Build system             | Tauri 2 + Rust + React + TypeScript + npm                      |
| Cross-platform CI status | Windows fixes recent; macOS/Linux assumed working              |
| Release readiness doc    | **None** — gate before joining the signing cluster             |
| Migration state          | Clean (no `legacy-origin` remote)                              |
| Audience profile         | Roguelike — strong itch.io fit                                 |
