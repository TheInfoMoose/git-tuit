## Problem Description
`filetreelist/src/{filetree.rs:70-197,filetreeitems.rs:140-151}`, `app.rs:885-892`, `revision_files.rs:246-251` — `select_file` miss bricks `FilesTab` nav; re-select hidden; sibling collapse icon bug; stale scrollbar — suggested fix: Keep selection on miss, always expand path, check `starts_with` before collapse, update `visual_selection`, scope fuzzy target
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-15`). Planned phase: 6.

## Reproduction Steps
1. Inspect `filetreelist/src/{filetree.rs:70-197,filetreeitems.rs:140-151}`, `app.rs:885-892`, `revision_files.rs:246-251` (static audit finding; see `AUDIT_REPORT.md` P2-15).
2. Observe: `select_file` miss bricks `FilesTab` nav; re-select hidden; sibling collapse icon bug; stale scrollbar

## Fix Log & Agent Notes
