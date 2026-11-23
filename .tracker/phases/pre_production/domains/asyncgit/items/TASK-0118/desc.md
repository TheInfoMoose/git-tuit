## Problem Description
STEP 10D changed worktree_matches_index so a missing gitlink dir no longer blocks abort (base refused). No test covers it: t10d_empty_submodule_dir_removed creates sub/ first and already passed on base code.
Found during: Phase 10 review.

## Reproduction Steps
In asyncgit/src/sync/merge.rs revert the is_absent arm of worktree_matches_index to return Ok(false); cargo test t10d_ still passes.

## Fix Log & Agent Notes
