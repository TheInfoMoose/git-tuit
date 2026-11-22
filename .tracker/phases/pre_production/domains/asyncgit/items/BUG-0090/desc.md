## Problem Description
abort_pending_state treats any directory as matching a gitlink index entry (asyncgit/src/sync/merge.rs:147), then calls remove_dir_all on merge-added paths (merge.rs:338). A submodule the merge added and the user initialized is deleted with its .git and any work in it. git reset --merge leaves submodule dirs in place.
Found during: Phase 9 review.

## Reproduction Steps
Merge that adds a submodule stops on a conflict; run git submodule update --init; abort merge -> submodule directory and its contents are gone.

## Fix Log & Agent Notes
