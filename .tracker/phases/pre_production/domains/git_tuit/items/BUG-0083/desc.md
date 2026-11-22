## Problem Description
`src/popups/reset.rs:137,156` (0aa3b194) — (a) a failed confirmed hard reset closes the picker, because `try_or_popup!` does not return. (b) the prompt gets the whole commit message as the subject (`get_commit_info`), so a message with a body shows fragments of the body and trailers — suggested fix: hide only on success, and pass the first line of the message.
Kind: bug
Review label: (a) spec gap: 8E 1e covers a failed discard only. (b) coder and validator miss: the field is the subject.
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Hard reset to a commit with a multi-line message and read the prompt.
2. Make the reset fail (hold index.lock) and confirm. The picker closes.

## Fix Log & Agent Notes
