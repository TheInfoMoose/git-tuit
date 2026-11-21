## Problem Description
`src/args.rs:71-82`, `src/ui/style.rs:293-324` — `-t`/`--theme` path traversal & arbitrary read/overwrite — suggested fix: Require single `Component::Normal` filename before joining to `confpath` + unit tests
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-18`). Planned phase: 4.

Parked at Phase 3 review (git-parity threat model): --theme is the user's own CLI argument.

## Reproduction Steps
1. Inspect `src/args.rs:71-82`, `src/ui/style.rs:293-324` (static audit finding; see `AUDIT_REPORT.md` P1-18).
2. Observe: `-t`/`--theme` path traversal & arbitrary read/overwrite

## Fix Log & Agent Notes
