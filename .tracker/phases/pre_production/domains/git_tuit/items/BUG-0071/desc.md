## Problem Description
`src/popups/checkout_option.rs` (`handle_event`) runs `discard_status` (hard reset) before `checkout()`. If the checkout then fails, the branch hasn't changed and the local changes are already gone. — suggested fix: make sure the switch can succeed before discarding, so that a failed switch changes nothing.
Kind: bug
Found during: Phase 8 planning (Claude, code trace, not executed). Audit section 4.1 row 2 (P2-06). BUG-0040 closed without this part.

Phase 8 review (Claude, code trace, not executed): NOT fixed by 72eea2cc (STEP 8B). `checkout_branch_discard` still runs the forced checkout before two steps that can refuse: `set_head` fails when the target is checked out in a linked worktree, and `set_upstream` fails when no fetch refspec matches the remote branch. In both cases the local changes are gone and HEAD has not moved. Check worktrees and create the branch with its upstream before discarding. Coder and validator miss, 8B item 1: "checks everything that can refuse the switch before it discards or creates anything".

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Edit a tracked file and create an untracked file that the target branch tracks.
2. Switch to that branch with "Discard all local changes".
3. Observe: the checkout fails on the untracked file and the edit is gone.

## Fix Log & Agent Notes
