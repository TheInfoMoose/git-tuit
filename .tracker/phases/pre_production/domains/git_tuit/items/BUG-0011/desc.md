## Problem Description
`deny.toml:15,26-62` — Legacy `version = 2` / `{ name = }` syntax & 3 stale `skip-tree` entries — suggested fix: Migrate to `cargo-deny 0.20` `{ crate = }`, prune stale entries, verify `bans`/`licenses` post-fetch
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-14`). Planned phase: 2.

## Reproduction Steps
1. Inspect `deny.toml:15,26-62` (static audit finding; see `AUDIT_REPORT.md` P0-14).
2. Observe: Legacy `version = 2` / `{ name = }` syntax & 3 stale `skip-tree` entries

## Fix Log & Agent Notes
