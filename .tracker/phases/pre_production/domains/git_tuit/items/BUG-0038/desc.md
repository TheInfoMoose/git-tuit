## Problem Description
`src/app.rs:381-1056`, `changes.rs:274`, `stashlist.rs:42`, `stashing.rs:264`, `push.rs:321`, `push_tags.rs:243`, `pull.rs:268`, `commit.rs:583` — Uncaught `?` errors terminate entire TUI — suggested fix: Wrap interactive ops in `try_or_popup!`; handle missing remote gracefully in `delete_tag`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-23`). Planned phase: 5.

## Reproduction Steps
1. Inspect `src/app.rs:381-1056`, `changes.rs:274`, `stashlist.rs:42`, `stashing.rs:264`, `push.rs:321`, `push_tags.rs:243`, `pull.rs:268`, `commit.rs:583` (static audit finding; see `AUDIT_REPORT.md` P1-23).
2. Observe: Uncaught `?` errors terminate entire TUI

## Fix Log & Agent Notes
