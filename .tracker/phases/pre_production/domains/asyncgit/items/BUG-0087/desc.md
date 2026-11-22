## Problem Description
check_branch_not_checked_out_elsewhere (asyncgit/src/sync/branch/mod.rs:535-550) opens the main repo via commondir and compares its HEAD with no is_bare() guard. Git skips bare worktrees here. No test covers the linked -> main direction.
Found during: Phase 9 review.

## Reproduction Steps
Bare repo with a linked worktree; from the linked worktree, discard-and-switch to the branch the bare HEAD names (e.g. main) -> refuses "branch main is checked out at <bare dir>". git switch allows it.

## Fix Log & Agent Notes
