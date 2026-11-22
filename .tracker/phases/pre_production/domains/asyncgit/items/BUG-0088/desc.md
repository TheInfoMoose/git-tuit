## Problem Description
apply_discard_checkout (asyncgit/src/sync/branch/mod.rs:659-683) calls cleanup_state after set_head. If cleanup fails, checkout_branch_discard returns Err and tries to delete the newly created branch, which HEAD now points to. libgit2 refuses and the error is swallowed. The repo has switched but the user sees a failure, and MERGE_HEAD may remain.
Found during: Phase 9 review.

## Reproduction Steps
Merge in progress, discard-and-switch to a remote branch, make .git/MERGE_HEAD undeletable -> error shown, but HEAD is on the new branch and the branch still exists.

## Fix Log & Agent Notes
