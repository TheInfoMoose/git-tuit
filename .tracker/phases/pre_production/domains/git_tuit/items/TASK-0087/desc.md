## Problem Description
`src/tabs/revlog.rs:517-527` — the revert key in the Log tab calls `revert_commit()` on the selection immediately, with no confirmation, unlike reset and discard — suggested fix: route it through the confirm popup, naming the commit.
Kind: bug
Found during: Phase 7 pilot review (Claude, read-only)

## Reproduction Steps
1. Select a commit in the Log tab and press the revert key.
2. Observe: the revert runs with no prompt.

## Fix Log & Agent Notes
