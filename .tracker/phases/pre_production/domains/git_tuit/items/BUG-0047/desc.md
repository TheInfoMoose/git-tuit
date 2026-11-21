## Problem Description
`status_tree.rs:320,485`, `revision_files.rs:488,496`, `tabs/{files.rs:66-101,status.rs:141-142}` — Closing Blame/FileRevlog hides underlying `Files`/`Status`/`Stashing` pane — suggested fix: Remove `self.hide()` back-channel or emit `NeedsUpdate::ALL` / restore visibility on `PopupStackPop`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-13`). Planned phase: 6.

## Reproduction Steps
1. Inspect `status_tree.rs:320,485`, `revision_files.rs:488,496`, `tabs/{files.rs:66-101,status.rs:141-142}` (static audit finding; see `AUDIT_REPORT.md` P2-13).
2. Observe: Closing Blame/FileRevlog hides underlying `Files`/`Status`/`Stashing` pane

## Fix Log & Agent Notes
