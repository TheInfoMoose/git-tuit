## Problem Description
`.github/workflows/ci.yml:284` — `cargo fmt -- --check` misses workspace crates — suggested fix: Change to `cargo fmt --all -- --check`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-06`). Planned phase: 2.

## Reproduction Steps
1. Inspect `.github/workflows/ci.yml:284` (static audit finding; see `AUDIT_REPORT.md` P0-06).
2. Observe: `cargo fmt -- --check` misses workspace crates

## Fix Log & Agent Notes
