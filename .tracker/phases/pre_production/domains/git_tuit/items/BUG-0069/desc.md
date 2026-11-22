## Problem Description
`src/components/commitlist.rs:383-399,309` — in highlight mode, Down targets the next match by its index in the search results. If that commit isn't in the list (not loaded yet during a reload, or rewritten away), `select_commit` fails and the app shows "Could not select commit" — suggested fix: move only among matches present in the list, and define what the key does when none is loaded.
The same code is in 7bbc5de8, so it predates STEP 7N.
Kind: bug
Found during: pilot root-cause check of STEP 7N (Claude, read-only; not reproduced).

## Reproduction Steps
From a code trace, not executed.
1. In the Log tab, run a search with at least two matches.
2. Amend or rebase from another terminal so that the second match no longer exists, and let the log reload.
3. With the first match selected (or none), press Down. Observe: the "Could not select commit" error popup.

## Fix Log & Agent Notes
