## Problem Description
`src/components/diff.rs:547-591,915-942`, `src/popups/confirm.rs` ForcePush arm — remainder of audit P2-07 not covered by STEP 5K: discarding lines with nothing selected still opens "discard 0 selected lines?", and the ForcePush confirmation truncates the branch name — suggested fix: don't queue Action::ResetLines when the selection is empty, and show the full branch name in the force-push message.
Found during: Phase 5 review (P2-07 remainder).

## Reproduction Steps
1. In the diff view, trigger "discard lines" with no lines selected → confirm says "0 selected lines".
2. Force-push a branch with a long or slashed name → the confirm text shows a shortened name.

## Fix Log & Agent Notes
