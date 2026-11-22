## Problem Description
`asyncgit/src/sync/branch/mod.rs:464` (`find_untracked_collisions`, 72eea2cc) — the collision check skips every untracked file whose path is staged as deleted or exists in HEAD. The forced checkout then overwrites that file, or deletes it when the target has no such path. D3 says refuse — suggested fix: count these paths as collisions when the target would write or remove them, and test both targets.
Kind: bug
Review label: coder and validator miss. 8B 1b: "An untracked file is in the target's way: refuse and name it."
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. `git rm --cached notes.txt`, then edit notes.txt.
2. Switch to another branch with "discard all local changes".
3. Expected from the code: the edited file is overwritten or deleted, with no refusal.

## Fix Log & Agent Notes
