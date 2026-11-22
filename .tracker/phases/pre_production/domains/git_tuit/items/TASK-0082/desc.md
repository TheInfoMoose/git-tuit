## Problem Description
STEP 7B spec gap: MUTATION_SPEC item 2 adds `pub fn is_empty(&self) -> bool` to `DiffComponent` (intended for STEP 7C in `src/tabs/status.rs`), but `gitui` is a binary crate with `dead_code` denied in `src/main.rs:39`. Because `is_empty` has no caller in `bin "gitui"` yet and `#[allow(...)]` is forbidden unless specified, `cargo clippy --workspace --all-targets -- -D warnings` fails with `error: method `is_empty` is never used`. Location: `src/components/diff.rs:270`.
Lapse: L4 other
Kind: scope-lapse
Found during: STEP 7B (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
