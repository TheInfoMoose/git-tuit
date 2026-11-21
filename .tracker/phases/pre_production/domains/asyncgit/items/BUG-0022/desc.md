## Problem Description
`asyncgit/src/sync/sign.rs:152-362` — Relative/PATH `gpg.*program`, non-`ssh-` literal key in argv/trace, missing `--`, lossy temp path — suggested fix: Reject relative-with-sep/`..` program, sanitize `PATH`, temp-file all SSH keys (`0o600`), validate `signingKey`, redact trace
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-09`). Planned phase: 4.

## Reproduction Steps
1. Inspect `asyncgit/src/sync/sign.rs:152-362` (static audit finding; see `AUDIT_REPORT.md` P1-09).
2. Observe: Relative/PATH `gpg.*program`, non-`ssh-` literal key in argv/trace, missing `--`, lossy temp path

## Fix Log & Agent Notes
