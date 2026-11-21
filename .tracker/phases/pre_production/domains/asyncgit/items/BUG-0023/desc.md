## Problem Description
`asyncgit/src/sync/hooks.rs:101-106,226-278`, `git2-hooks/src/lib.rs:90-108` — `pre-push` stdin refspec newline/whitespace injection — suggested fix: Validate `local_ref`/`remote_ref` via `Reference::is_valid_name` & reject whitespace/controls
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-10`). Planned phase: 4.

## Reproduction Steps
1. Inspect `asyncgit/src/sync/hooks.rs:101-106,226-278`, `git2-hooks/src/lib.rs:90-108` (static audit finding; see `AUDIT_REPORT.md` P1-10).
2. Observe: `pre-push` stdin refspec newline/whitespace injection

## Fix Log & Agent Notes
