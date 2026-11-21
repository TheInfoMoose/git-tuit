## Problem Description
`asyncgit/src/sync/utils.rs:182-194` — `repo_write_file` path traversal & symlink truncation — suggested fix: Reject absolute/`..`, canonicalize & assert `starts_with`, refuse symlinks, atomic write-rename
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-11`). Planned phase: 4.

## Reproduction Steps
1. Inspect `asyncgit/src/sync/utils.rs:182-194` (static audit finding; see `AUDIT_REPORT.md` P1-11).
2. Observe: `repo_write_file` path traversal & symlink truncation

## Fix Log & Agent Notes
