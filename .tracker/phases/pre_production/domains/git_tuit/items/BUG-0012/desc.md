## Problem Description
`.clippy.toml:1-2`, `asyncgit/src/revlog.rs:370,403` — >1,270 `#[cfg(test)]` unwrap/expect/panic errors under `--all-targets` + `unit_cmp` — suggested fix: Enable `allow-{unwrap,expect,panic}-in-tests = true` in `.clippy.toml`; fix all Clippy diagnostics
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-15`). Planned phase: 3.

## Reproduction Steps
1. Inspect `.clippy.toml:1-2`, `asyncgit/src/revlog.rs:370,403` (static audit finding; see `AUDIT_REPORT.md` P0-15).
2. Observe: >1,270 `#[cfg(test)]` unwrap/expect/panic errors under `--all-targets` + `unit_cmp`

## Fix Log & Agent Notes
