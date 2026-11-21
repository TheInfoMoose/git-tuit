## Problem Description
`git2-hooks/src/lib.rs:150-176` — 4 prod `.unwrap()` & `chmod +x` subprocess in `create_hook` — suggested fix: Use `Permissions::from_mode(0o700)` & `Result` (`try_create_hook` if keeping semver)
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-08`). Planned phase: 4.

Parked at Phase 3 review (git-parity threat model): create_hook is only used by tests.

## Reproduction Steps
1. Inspect `git2-hooks/src/lib.rs:150-176` (static audit finding; see `AUDIT_REPORT.md` P1-08).
2. Observe: 4 prod `.unwrap()` & `chmod +x` subprocess in `create_hook`

## Fix Log & Agent Notes
