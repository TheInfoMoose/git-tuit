## Problem Description
`src/tabs/revlog.rs:128-165,280-290`, `src/tabs/files.rs:50-59` — Search softlock & stuck `(0%)` syntax highlighting when switching tabs — suggested fix: Handle `CommitFilter` & `SyntaxHighlighting` updates even while hidden (or drain on `show()`)
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-14`). Planned phase: 6.

## Reproduction Steps
1. Inspect `src/tabs/revlog.rs:128-165,280-290`, `src/tabs/files.rs:50-59` (static audit finding; see `AUDIT_REPORT.md` P2-14).
2. Observe: Search softlock & stuck `(0%)` syntax highlighting when switching tabs

## Fix Log & Agent Notes
