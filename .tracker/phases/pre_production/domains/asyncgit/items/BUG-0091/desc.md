## Problem Description
worktree_matches_tree_entry in asyncgit/src/sync/branch/mod.rs reads each untracked-in-HEAD file fully into memory (std::fs::read) before hashing, on the UI thread.
Found during: Phase 10 review.

## Reproduction Steps
git rm --cached a multi-hundred-MB tracked binary, then discard-and-switch to another branch; memory spikes by the file size before the switch.

## Fix Log & Agent Notes
