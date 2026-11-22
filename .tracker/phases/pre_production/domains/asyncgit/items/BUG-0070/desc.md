## Problem Description
`asyncgit/src/sync/merge.rs` (`abort_pending_state`) calls `reset_workdir(repo, "*")`, and `asyncgit/src/sync/reset.rs` (`reset_workdir`) checks out with `remove_untracked(true).force()`. So aborting a merge or revert deletes every untracked, non-ignored file in the worktree. The prompt (`src/strings.rs:194-196`) only says uncommitted changes are reverted. `git merge --abort` (reset --merge) doesn't do this. — suggested fix: remove only what the merge or revert brought in. Whether unstaged edits also survive, as in git, is a product call.
Kind: bug
Found during: Phase 8 planning (Claude, code trace, not executed). Audit section 4.1 rows 15-16 (P2-07). BUG-0041 closed without this part, and BUG-0062 didn't carry it.

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Create an untracked file `notes.txt`.
2. Start a merge that conflicts in another file.
3. Abort it from the Status tab and confirm.
4. Observe: `notes.txt` is expected to be gone.

## Fix Log & Agent Notes
