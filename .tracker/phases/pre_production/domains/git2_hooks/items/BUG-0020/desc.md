## Problem Description
`git2-hooks/src/lib.rs:195-204,297-330` — `COMMIT_EDITMSG` symlink follow, `0o644`, no cleanup, lossy path, unbounded read — suggested fix: Non-symlink check + atomic `create_new` + `0o600` + RAII cleanup + bounded read + `OsStr` hook args
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-07`). Planned phase: 4.

## Reproduction Steps
1. Inspect `git2-hooks/src/lib.rs:195-204,297-330` (static audit finding; see `AUDIT_REPORT.md` P1-07).
2. Observe: `COMMIT_EDITMSG` symlink follow, `0o644`, no cleanup, lossy path, unbounded read

## Fix Log & Agent Notes
