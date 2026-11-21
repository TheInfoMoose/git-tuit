## Problem Description
`revlog.rs:482-522`, `branchlist.rs:348-378`, `commitlist.rs:860-865`, `stashlist.rs:195`, `submodules.rs:186-205`, `status.rs:612-619,899-908`, `remotelist.rs:142-145` — Missing confirmations on revert, merge, rebase, detached-HEAD checkout (leaking to `StashList`), `PushTags`, submodule update; `Undo` during rebase — suggested fix: Add `ConfirmPopup` gates, block checkout in `StashList`, guard `Undo` during rebase, allow force-push when behind
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-08`). Planned phase: 5.

## Reproduction Steps
1. Inspect `revlog.rs:482-522`, `branchlist.rs:348-378`, `commitlist.rs:860-865`, `stashlist.rs:195`, `submodules.rs:186-205`, `status.rs:612-619,899-908`, `remotelist.rs:142-145` (static audit finding; see `AUDIT_REPORT.md` P2-08).
2. Observe: Missing confirmations on revert, merge, rebase, detached-HEAD checkout (leaking to `StashList`), `PushTags`, submodule update; `Undo` during rebase

## Fix Log & Agent Notes
