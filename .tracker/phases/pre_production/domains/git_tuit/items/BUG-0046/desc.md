## Problem Description
`msg.rs`, `push*.rs`, `pull.rs`, `fetch.rs`, `checkout_option.rs`, `inspect_commit.rs`, `compare_commits.rs`, `file_revlog.rs`, `create_remote.rs`, `remotelist.rs`, `branchlist.rs`, `tag_commit.rs`, `log_search.rs`, `options.rs`, `strings.rs:1592` — 15 popup key-hint/`event()` mismatches, `log_find_commit` wrong key, `cmd_bar_toggle` swallowed — suggested fix: Align hints, `enabled`/`available` flags & `event()` guards; fix `Back [Esc]` on diff popups; pass through `.`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-12`). Planned phase: 6.

## Reproduction Steps
1. Inspect `msg.rs`, `push*.rs`, `pull.rs`, `fetch.rs`, `checkout_option.rs`, `inspect_commit.rs`, `compare_commits.rs`, `file_revlog.rs`, `create_remote.rs`, `remotelist.rs`, `branchlist.rs`, `tag_commit.rs`, `log_search.rs`, `options.rs`, `strings.rs:1592` (static audit finding; see `AUDIT_REPORT.md` P2-12).
2. Observe: 15 popup key-hint/`event()` mismatches, `log_find_commit` wrong key, `cmd_bar_toggle` swallowed

## Fix Log & Agent Notes
