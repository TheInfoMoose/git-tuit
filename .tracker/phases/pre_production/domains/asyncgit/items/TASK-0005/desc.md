## Problem Description
`asyncgit/Cargo.toml:41`, `Cargo.lock:3318,2942` — `scc 2.3.4` (`RUSTSEC-2026-0205`) & `pin-utils` (`RUSTSEC-2026-0441`) via `serial_test` — suggested fix: Disable `serial_test` default features (`futures-util`/`pin-utils`) & update `scc >= 3.8.4`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-13`). Planned phase: 2.

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
