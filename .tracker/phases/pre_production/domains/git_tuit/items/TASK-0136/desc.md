## Problem Description
STEP 13C spec gap: FORBIDDEN_SCOPE requires `Theme::init` to keep its `&PathBuf` signature (`pub fn init(theme_path: &PathBuf) -> Self`) and repo-rules ban `#[allow(...)]`, while MUTATION_SPEC delegates `init` to `fn init_for(theme_path: &Path, colorterm: Option<&str>) -> Self`. Once `init` delegates directly to `init_for`, `cargo clippy --workspace --all-targets -- -D warnings` fails with `clippy::ptr_arg` (implied by `#![deny(clippy::all)]` in `src/main.rs:43`) demanding `&Path` instead of `&PathBuf`. Location: `src/ui/style.rs:657`.
Lapse: L4 other
Kind: scope-lapse
Found during: STEP 13C (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
