## Problem Description
`src/args.rs:52-69,177-192` — Unvalidated `-d`/`-w` CLI/env paths — suggested fix: Reject empty/NUL `-d`/`-w`, verify & canonicalize `-w` directory
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-04`). Planned phase: 4.

Parked at Phase 3 review (git-parity threat model): -d/-w are the user's own CLI arguments.

## Reproduction Steps
1. Inspect `src/args.rs:52-69,177-192` (static audit finding; see `AUDIT_REPORT.md` P2-04).
2. Observe: Unvalidated `-d`/`-w` CLI/env paths

## Fix Log & Agent Notes
