## Problem Description
`create_branch.rs`, `rename_branch.rs`, `tag_commit.rs`, `create_remote.rs`, `rename_remote.rs`, `update_remote_url.rs`, `commit.rs`, `stashmsg.rs`, `goto_line.rs`, `submodules.rs` — Missing pre-submit ref/input validation, lost input on error, stale input on `Esc`, raw `{:?}` errors — suggested fix: Validate ref/URL/non-empty inputs before submit, keep popup+text on error, clear on `open()`, format user-friendly statuses
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-23`). Planned phase: 6.

## Reproduction Steps
1. Inspect `create_branch.rs`, `rename_branch.rs`, `tag_commit.rs`, `create_remote.rs`, `rename_remote.rs`, `update_remote_url.rs`, `commit.rs`, `stashmsg.rs`, `goto_line.rs`, `submodules.rs` (static audit finding; see `AUDIT_REPORT.md` P2-23).
2. Observe: Missing pre-submit ref/input validation, lost input on error, stale input on `Esc`, raw `{:?}` errors

## Fix Log & Agent Notes
