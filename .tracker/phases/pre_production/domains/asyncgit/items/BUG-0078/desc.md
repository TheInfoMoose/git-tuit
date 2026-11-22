## Problem Description
`asyncgit/src/sync/branch/mod.rs:652` (`checkout_branch_discard`, 72eea2cc) — a forced `checkout_tree` replaced the hard reset and does neither of the other two things the reset did. (a) It only rewrites index entries it acts on, so a staged edit whose worktree copy equals HEAD, and a staged new file deleted from disk, stay staged on the target branch. (b) It never clears MERGE_HEAD or other pending state, so the next commit on the target branch becomes a merge commit. Regression from 8B — suggested fix: after the checks, set the index to the target tree and clean up the repo state. Redesign this with BUG-0071.
Kind: bug
Review label: (a) coder and validator miss, 8B 1a: "staged and unstaged tracked changes are discarded". (b) spec gap: the pending state is not among the 8B writers.
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. (a) Stage an edit, put the HEAD content back in the worktree, discard-and-switch. The edit is still staged.
2. (b) Start a conflicting merge, discard-and-switch. MERGE_HEAD is still present on the target branch.

## Fix Log & Agent Notes
