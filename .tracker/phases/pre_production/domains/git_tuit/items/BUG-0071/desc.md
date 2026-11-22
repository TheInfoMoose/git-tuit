## Problem Description
`src/popups/checkout_option.rs` (`handle_event`) runs `discard_status` (hard reset) before `checkout()`. If the checkout then fails, the branch hasn't changed and the local changes are already gone. — suggested fix: make sure the switch can succeed before discarding, so that a failed switch changes nothing.
Kind: bug
Found during: Phase 8 planning (Claude, code trace, not executed). Audit section 4.1 row 2 (P2-06). BUG-0040 closed without this part.

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Edit a tracked file and create an untracked file that the target branch tracks.
2. Switch to that branch with "Discard all local changes".
3. Observe: the checkout fails on the untracked file and the edit is gone.

## Fix Log & Agent Notes
