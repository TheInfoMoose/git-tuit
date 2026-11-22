## Problem Description
`src/popups/push.rs:184` always passes `PrePushTarget::Branch`, including when `push_type` is a tag, and `asyncgit/src/sync/hooks.rs:281-291` has no single-tag target. A pre-push hook sees a branch ref when a remote tag is deleted. — suggested fix: add a tag target and use it for tag pushes and deletes.
Kind: bug
Found during: Phase 8 planning (Claude, code trace, not executed). Audit section 4.1 row 11. BUG-0041 closed without this part.

## Reproduction Steps
From a code trace, not executed.
1. Install a pre-push hook that logs stdin.
2. Delete tag `v1` locally and confirm deleting it on the remote.
3. Observe: the hook gets a `refs/heads/v1` line.

## Fix Log & Agent Notes
