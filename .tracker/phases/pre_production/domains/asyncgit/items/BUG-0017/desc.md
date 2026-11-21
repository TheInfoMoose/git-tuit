## Problem Description
`asyncgit/src/sync/cred.rs:16-22`, `remotes/callbacks.rs:185-187` — `#[derive(Debug)]` & raw URL logging leak passwords/tokens — suggested fix: Manual `Debug` redaction + `Drop` zeroing on `BasicAuthCredential`; redact URL userinfo in `callbacks.rs`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-04`). Planned phase: 4.

## Reproduction Steps
1. Inspect `asyncgit/src/sync/cred.rs:16-22`, `remotes/callbacks.rs:185-187` (static audit finding; see `AUDIT_REPORT.md` P1-04).
2. Observe: `#[derive(Debug)]` & raw URL logging leak passwords/tokens

## Fix Log & Agent Notes
