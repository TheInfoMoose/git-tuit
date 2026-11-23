## Problem Description
BUG-0091 fix (Oid::hash_file in worktree_matches_tree_entry) has no test that fails if it reverts to std::fs::read + hash_object; t11c_large_* (1 MiB + 1 byte) pass on both. Found during: Phase 11 review (F2).
Found during: Phase 11 review.

## Reproduction Steps
Revert asyncgit/src/sync/branch/mod.rs worktree_matches_tree_entry to std::fs::read + Oid::hash_object; run cargo test -p asyncgit t11c_; all pass.

## Fix Log & Agent Notes
