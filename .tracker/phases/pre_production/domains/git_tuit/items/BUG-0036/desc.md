## Problem Description
`src/popups/externaleditor.rs:69-118` — Ignores non-zero editor exit (`:cq`) & empty `GIT_EDITOR=""` — suggested fix: Skip empty env vars & abort cleanly on non-zero `ExitStatus`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-05`). Planned phase: 4.

## Reproduction Steps
1. Inspect `src/popups/externaleditor.rs:69-118` (static audit finding; see `AUDIT_REPORT.md` P2-05).
2. Observe: Ignores non-zero editor exit (`:cq`) & empty `GIT_EDITOR=""`

## Fix Log & Agent Notes
