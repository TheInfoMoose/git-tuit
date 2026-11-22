## Problem Description
`asyncgit/src/revlog.rs` (`AsyncLog::fetch`) runs `fetch_helper(..).expect("failed to fetch")` in a rayon worker. Any walk error (missing object, corrupt pack) panics there. gitui's panic hook (`src/main.rs:351-355`) handles every thread as a fatal crash, and `pending` is never cleared. — suggested fix: store the error, clear `pending`, notify, and show the error in the Log tab.
Kind: bug
Found during: Phase 8 planning (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Delete an object file that history needs.
2. Open the Log tab.
3. Observe: the fatal panic message is expected instead of an error popup.

## Fix Log & Agent Notes
