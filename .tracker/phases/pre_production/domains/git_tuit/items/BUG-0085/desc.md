## Problem Description
`src/popups/inspect_commit.rs:113`, `src/popups/compare_commits.rs:85` (ec97ba13) — the Close hint always lists Left, but on a nested file or an expanded folder the file tree consumes Left to move or collapse — suggested fix: ask the tree whether it would consume Left before adding it to the hint.
Kind: bug
Review label: coder and validator miss. 8L 3b: "Left is shown as close where it closes".
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Inspect a commit, select a file inside a folder. The hint says Left closes, and Left moves to the folder.

## Fix Log & Agent Notes
