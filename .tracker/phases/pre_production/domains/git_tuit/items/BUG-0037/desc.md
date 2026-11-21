## Problem Description
`src/app.rs:1060-1086` — `delete_remote_branch` truncates `feat/foo` & drops `<remote>` — suggested fix: Parse `(remote, full_branch)` from `refs/remotes/<remote>/<branch>`, push delete to `<remote>` (+ `STEP 3A` remote param)
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-22`). Planned phase: 5.

## Reproduction Steps
1. Inspect `src/app.rs:1060-1086` (static audit finding; see `AUDIT_REPORT.md` P1-22).
2. Observe: `delete_remote_branch` truncates `feat/foo` & drops `<remote>`

## Fix Log & Agent Notes
