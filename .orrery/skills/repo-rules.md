# git-tuit repo rules

git-tuit is a fork of gitui (Rust TUI for git). Workspace crates: the root `gitui` binary, `asyncgit`, `filetreelist`, `git2-hooks`, `git2-testing`, `scopetime`, `invalidstring`.

**Read first:** `/workspace/CLAUDE.md` (workflow, tracking) and the step block you were given in `STEPPED_IMPLEMENTATION_PLAN.md`. `AUDIT_REPORT.md` explains each finding ID (`P0-01` … `P3-08`) a step cites.

## Must not change
- The binary/crate name `gitui`, the `~/.config/gitui/` config path, and the format of existing `key_bindings.ron`, `key_symbols.ron` and `theme.ron` files (existing user files must still load).
- Forbidden paths (never edit): `AUDIT_REPORT.md`, `STEPPED_IMPLEMENTATION_PLAN.md`, `.orrery/`, `.phased/`, `scripts/`, `wix/`. `.tracker/`: only new **pending** items you file.
- No commits. The pass hook commits.

## Style
- `rustfmt.toml`: `hard_tabs = true`, `max_width = 70`, Unix newlines. Run `cargo fmt --all` before reporting.
- `.clippy.toml`: `msrv = "1.88.0"`. Workspace crates deny `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `pedantic`, `nursery`. Don't add `#[allow(...)]` unless the step says to.
- Test data uses placeholders only (`192.0.2.0/24`, `example.com`, `testuser`).

## Gates
The step's POST_GATE starts with the plan header's **Base gate** line. Run the whole POST_GATE line exactly as written before reporting `Complete`. If it fails for a reason outside ALLOWED_FILES, report `Blocked` with the failing output instead of editing other files.
