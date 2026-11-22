## Problem Description
`src/components/syntax_text.rs:103-110` (`load_file`) treats a file as loaded when only its path matches, and `src/components/revision_files.rs` (`set_commit`, `refresh_files`) doesn't clear the viewer on a successful revision change. The Files tab and the revision-files popup can show one revision's content under another revision's title. — suggested fix: key the loaded file by blob or revision, and clear it when the tree is replaced.
Kind: bug
Found during: Phase 8 planning (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed.
1. Open a file in the Files tab.
2. Commit a change to it from another terminal.
3. Select that file again. Observe: the old content is shown.

## Fix Log & Agent Notes
