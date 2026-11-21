## Problem Description
`deny.toml`, `build.rs:8-47` — Missing `[sources]`/`[graph]`, `build.rs` slice/unwrap & unconditional warnings — suggested fix: Add `[sources]` + Linux `[graph] targets` to `deny.toml`; guard `build.rs` slice/parse if in scope
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-02`). Planned phase: 2.

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
