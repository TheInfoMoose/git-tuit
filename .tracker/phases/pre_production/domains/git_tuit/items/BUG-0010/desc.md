## Problem Description
`Cargo.lock:2470` — `libz-sys 1.1.21` (`RUSTSEC-2026-0418` + `RUSTSEC-2026-0544`) — suggested fix: Update `libz-sys` to `>= 1.1.33` cleanly in `Cargo.lock`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-10`). Planned phase: 2.

## Reproduction Steps
1. Inspect `Cargo.lock:2470` (static audit finding; see `AUDIT_REPORT.md` P0-10).
2. Observe: `libz-sys 1.1.21` (`RUSTSEC-2026-0418` + `RUSTSEC-2026-0544`)

## Fix Log & Agent Notes
