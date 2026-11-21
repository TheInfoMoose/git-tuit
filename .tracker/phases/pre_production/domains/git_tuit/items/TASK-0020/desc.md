## Problem Description
STEP 2A spec gap: POST_GATE runs `cargo fmt --all -- --check`, which selects `stable` (`1.100.0`) via `rust-toolchain.toml` and fails on 4 pre-existing Rust files outside `ALLOWED_FILES` (`.github/workflows/ci.yml`): `asyncgit/src/sync/reword.rs:32`, `src/app.rs:1182`, `src/components/commitlist.rs:354`, `src/popups/branchlist.rs:418`. Note `RUSTUP_TOOLCHAIN=1.99.1 cargo fmt --all -- --check` exits 0. Location: `asyncgit/src/sync/reword.rs:32`.
Lapse: L2 file outside ALLOWED_FILES
Kind: scope-lapse
Found during: STEP 2A (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
