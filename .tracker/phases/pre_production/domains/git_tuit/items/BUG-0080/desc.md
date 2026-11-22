## Problem Description
`src/popups/confirm.rs:274` (`ConfirmGuard::PushRemote`, 0d5c04ae) — the force-push guard checks only the remote. A branch tip that moved while the prompt was open is force-pushed anyway. D2 says refuse when the named target changed — suggested fix: pin the branch tip when the prompt opens and compare it on Enter.
Kind: bug
Review label: spec gap: 8F lists no writers of the branch being pushed.
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Open the force-push confirmation.
2. Move the branch from another terminal.
3. Press Enter. The new tip is force-pushed.

## Fix Log & Agent Notes
