## Problem Description
`git2-hooks/src/hookspath.rs:208-256` — Unix `sh` missing `--` option terminator; brittle Windows `sh -c` — suggested fix: Pass `--` on Unix (`sh -- <hook>`) and positional `"$0" "$@"` on Windows
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-06`). Planned phase: 4.

## Reproduction Steps
1. Inspect `git2-hooks/src/hookspath.rs:208-256` (static audit finding; see `AUDIT_REPORT.md` P1-06).
2. Observe: Unix `sh` missing `--` option terminator; brittle Windows `sh -c`

## Fix Log & Agent Notes
