## Problem Description
`asyncgit/src/sync/cred.rs:50-159`, `remotes/{callbacks.rs:179-220,mod.rs:302-334,push.rs,tags.rs}`, `hooks.rs` — `pushurl` mismatch, loose `starts_with("http")`, `fetch_all` cred broadcast, unbound `Callbacks` — suggested fix: Check `pushurl`, parse HTTP/HTTPS scheme, bind `Callbacks::with_expected_url`, per-remote creds in `fetch_all`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-03`). Planned phase: 4.

## Reproduction Steps
1. Inspect `asyncgit/src/sync/cred.rs:50-159`, `remotes/{callbacks.rs:179-220,mod.rs:302-334,push.rs,tags.rs}`, `hooks.rs` (static audit finding; see `AUDIT_REPORT.md` P1-03).
2. Observe: `pushurl` mismatch, loose `starts_with("http")`, `fetch_all` cred broadcast, unbound `Callbacks`

## Fix Log & Agent Notes
