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

## Outcome steps (Phase 7 pilot: STEP 7J-7M)
For a step whose MUTATION_SPEC starts "Outcome step", these rules override the neon-coder and phased-validator rules about exact wording, named code and "nothing extra".

**Coder**
- Each item is a behavior. You choose the names, signatures and code, anywhere inside ALLOWED_FILES. Choosing an implementation is not ambiguity and is not a scope lapse.
- You may add private or `pub(crate)` helpers, imports and small refactors that an item needs. A helper lands with its caller. No new dependencies.
- For each `Proof: test`, write that test first with exactly that name and see it fail on the old behavior. Extract a small pure helper if the component is hard to construct, and make the real code path call it. Report the failing assertion for each test. If a test can't fail on the old behavior, say so in the report.
- Report `Blocked` only if an outcome is impossible inside ALLOWED_FILES, contradicts another outcome, or needs a product decision.
- Still banned: `#[allow]`, weakened or deleted tests, swallowed errors, stubs, files outside ALLOWED_FILES.

**Validator** (replaces checks 1, 2, 4 and 7; checks 3, 5 and 6 still apply)
- 1: Is each behavior true on the real code path? Trace one concrete input per item, including empty, loading and error states where they apply. Don't compare wording.
- 2: Does every change serve one of the items, with no new dependency?
- 4: Does each named test exercise production code, and would it fail on `git show HEAD:<file>`? Deny a test that would pass on the old code.
- 7: Not applicable. These steps have no source-text markers.
- In `feedback`, give the failing input, what happens and what should happen.
