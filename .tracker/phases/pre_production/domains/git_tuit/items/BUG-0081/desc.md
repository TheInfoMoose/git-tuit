## Problem Description
`src/popups/commit.rs:667` — when `can_commit()` is false, Ctrl+D falls through to the text area as delete-next-char. With the cursor at the start of a comments-only message or an unchanged template, the first press deletes a character (the `#`) and the second commits it — suggested fix: consume the commit key whether or not commit is enabled.
Kind: bug
Review label: coder and validator miss. 8J 1b: "Only comment lines, using core.commentChar: blocked." Older for templates (1c).
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Set a commit.template whose first line is `# note`, stage a file, open commit.
2. Press Ctrl+D twice. A commit is created.

## Fix Log & Agent Notes
