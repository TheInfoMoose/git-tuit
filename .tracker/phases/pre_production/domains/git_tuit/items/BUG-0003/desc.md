## Problem Description
`.github/workflows/ci.yml:68-70` — `make clippy` skips tests, omits `-D warnings`, collides `regex-*` — suggested fix: Run `cargo clippy --workspace --all-targets -- -D warnings` directly
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-03`). Planned phase: 2.

## Reproduction Steps
1. Inspect `.github/workflows/ci.yml:68-70` (static audit finding; see `AUDIT_REPORT.md` P0-03).
2. Observe: `make clippy` skips tests, omits `-D warnings`, collides `regex-*`

## Fix Log & Agent Notes
