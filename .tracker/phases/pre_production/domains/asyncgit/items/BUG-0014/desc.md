## Problem Description
`asyncgit/src/revlog.rs:391-401`, `asyncgit/src/sync/sign.rs:755,870,907,991` — Process-wide `GIT_DIR`/`GNUPGHOME` env races/leaks under `cargo test` — suggested fix: Wrap env mutations in RAII `Drop` guards and fix any test failures post-fetch
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-17`). Planned phase: 3.

## Reproduction Steps
1. Inspect `asyncgit/src/revlog.rs:391-401`, `asyncgit/src/sync/sign.rs:755,870,907,991` (static audit finding; see `AUDIT_REPORT.md` P0-17).
2. Observe: Process-wide `GIT_DIR`/`GNUPGHOME` env races/leaks under `cargo test`

## Fix Log & Agent Notes
