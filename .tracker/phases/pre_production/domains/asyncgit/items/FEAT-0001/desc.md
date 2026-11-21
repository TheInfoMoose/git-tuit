## Problem Description
`asyncgit/src/sync/config.rs:111-140`, `asyncgit/src/sync/mod.rs` — No global-only config reader (`Config::open_default`) — suggested fix: Add `get_global_config_string` for `externaleditor.rs` & `commit.rs`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-12`). Planned phase: 4.

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
