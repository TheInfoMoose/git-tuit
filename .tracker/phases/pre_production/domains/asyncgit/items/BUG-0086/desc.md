## Problem Description
`asyncgit/src/revlog.rs:139` (`head_changed`) — an unborn HEAD reports no change, so the Log keeps the old history and 8O never drops those marks — suggested fix: treat born to unborn as a change.
Kind: bug
Review label: spec gap: 8O 1d names empty history, but the writer that produces it is outside its allowed files.
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Mark a commit, run `git checkout --orphan empty` in another terminal, refresh. History and mark remain.

## Fix Log & Agent Notes
