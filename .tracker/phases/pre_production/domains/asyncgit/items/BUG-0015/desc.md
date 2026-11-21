## Problem Description
`asyncgit/src/sync/cred.rs:100-175` — Repo `.git/config` `credential.helper` RCE & `credential.username` newline injection — suggested fix: Use `Config::open_default()` only; validate/strip URL userinfo via `url::Url`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-02`). Planned phase: 4.

## Reproduction Steps
1. Inspect `asyncgit/src/sync/cred.rs:100-175` (static audit finding; see `AUDIT_REPORT.md` P1-02).
2. Observe: Repo `.git/config` `credential.helper` RCE & `credential.username` newline injection

## Fix Log & Agent Notes
