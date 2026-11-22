## Problem Description
`asyncgit/src/sync/commit.rs:180-187` (`commit_message_prettify`) — the first character of the config value is used, so `auto` makes `a` the comment character and lines starting with `a` are dropped from the message. Since 8J a message made only of such lines is also blocked — suggested fix: handle `auto`, or fall back to `#` for values that are not one character.
Kind: bug
Review label: older bug, outside Phase 8.
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. `git config core.commentChar auto`
2. Commit with the message `add parser`. It is treated as empty.

## Fix Log & Agent Notes
