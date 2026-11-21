## Problem Description
`revision_files.rs:123-145`, `taglist.rs:228-260`, `inspect_commit.rs:174-192`, `fuzzy_find.rs:110-129,336-375`, `stashlist.rs:199-221`, `app.rs:803` — Modal event leaks, `FuzzyFind` `Esc`/double-nav bug, `TabSwitchStatus` stale cmdbar — suggested fix: Return `Consumed` in modals & `StashList`; restore/early-return in `FuzzyFind`; update cmdbar on `TabSwitchStatus`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-11`). Planned phase: 6.

## Reproduction Steps
1. Inspect `revision_files.rs:123-145`, `taglist.rs:228-260`, `inspect_commit.rs:174-192`, `fuzzy_find.rs:110-129,336-375`, `stashlist.rs:199-221`, `app.rs:803` (static audit finding; see `AUDIT_REPORT.md` P2-11).
2. Observe: Modal event leaks, `FuzzyFind` `Esc`/double-nav bug, `TabSwitchStatus` stale cmdbar

## Fix Log & Agent Notes
