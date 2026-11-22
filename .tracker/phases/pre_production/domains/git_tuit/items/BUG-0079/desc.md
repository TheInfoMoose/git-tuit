## Problem Description
`src/popups/push.rs:174` (`start_push`) — credentials always come from the default push remote, but the push can go to another remote, and the credential callback does not check the host, so one host can receive the password stored for another. Regression in 0d5c04ae for remote-tag delete (`src/app.rs:1264` picks the default remote and pins it). Older paths with the same cause: branch push to an upstream that differs from pushRemote or pushDefault, remote-branch delete, and push tags (`extract_username_password` reads `url`, the push uses `pushurl`) — suggested fix: resolve the remote first, then take the need-credential check and the credentials from that remote push URL.
Kind: bug
Review label: spec gap: 8F lists the writers of the push remote, not the credentials that depend on it.
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. origin = https://git.example.com/r.git, fork = https://fork.example.org/r.git, remote.pushDefault = fork.
2. Store a credential for fork only, delete a local tag, confirm the remote delete.
3. Expected from the code: the fork credential is offered to git.example.com.

## Fix Log & Agent Notes
