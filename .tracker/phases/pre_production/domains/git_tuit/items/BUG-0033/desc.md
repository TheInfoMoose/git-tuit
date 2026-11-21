## Problem Description
`src/popups/commit.rs:164-200,400-424` — `COMMIT_EDITMSG` symlink/`0o644` & `commit.template` `shellexpand::full()` RCE/read — suggested fix: Atomic `0o600` non-symlink edit file; `shellexpand::tilde` / global-only bounded read for template
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-21`). Planned phase: 4.

## Reproduction Steps
1. Inspect `src/popups/commit.rs:164-200,400-424` (static audit finding; see `AUDIT_REPORT.md` P1-21).
2. Observe: `COMMIT_EDITMSG` symlink/`0o644` & `commit.template` `shellexpand::full()` RCE/read

## Fix Log & Agent Notes
