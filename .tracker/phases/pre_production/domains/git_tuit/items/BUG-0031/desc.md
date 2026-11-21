## Problem Description
`src/args.rs:195-227` — `setup_logging` creates `0o644` log & follows symlinks — suggested fix: Reject symlinks, open with `mode(0o600)`, enforce `0o700` on new cache dir
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-19`). Planned phase: 4.

## Reproduction Steps
1. Inspect `src/args.rs:195-227` (static audit finding; see `AUDIT_REPORT.md` P1-19).
2. Observe: `setup_logging` creates `0o644` log & follows symlinks

## Fix Log & Agent Notes
