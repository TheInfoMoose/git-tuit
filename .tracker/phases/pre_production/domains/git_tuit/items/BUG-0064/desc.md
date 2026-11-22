## Problem Description
`src/components/commitlist.rs:92-99,229` — regression from d5fd91ec (STEP 7M): `clear()` and `refresh_extend_data()` clamp the selection while history is empty or still streaming. `Revlog::update` calls `clear()` on every HEAD change (src/tabs/revlog.rs:130-135), so the selection jumps to the newest commit. Revert in the log runs on the selection with no confirmation, so it can hit the wrong commit. t7m_commit_selection_clamps asserts the bad reset — suggested fix: clamp only once loading has finished; replace that assert with a reload test.
Kind: bug
Found during: Phase 7 pilot review (Claude, read-only; not reproduced)

## Reproduction Steps
1. In the Log tab select the 10th commit.
2. Change HEAD (commit from another terminal).
3. Observe: the selection is on the newest commit.

## Fix Log & Agent Notes
