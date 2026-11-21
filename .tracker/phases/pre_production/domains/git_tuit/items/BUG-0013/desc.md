## Problem Description
`Makefile:2,21-110` — `Makefile` hard-requires `cargo-nextest`, misses `--all` on `fmt` & `--all-targets` on `clippy` — suggested fix: Align `Makefile` targets with Linux CI & `cargo test --workspace` parity
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-16`). Planned phase: 2.

## Reproduction Steps
1. Inspect `Makefile:2,21-110` (static audit finding; see `AUDIT_REPORT.md` P0-16).
2. Observe: `Makefile` hard-requires `cargo-nextest`, misses `--all` on `fmt` & `--all-targets` on `clippy`

## Fix Log & Agent Notes
