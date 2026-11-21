## Problem Description
`src/tabs/stashing.rs:72-80,229-273` — `Stashing` list ignores `stash_untracked` & `keep_index` toggles — suggested fix: Filter `StatusParams` from toggle state & preserve options cleanly
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-17`). Planned phase: 6.

## Reproduction Steps
1. Inspect `src/tabs/stashing.rs:72-80,229-273` (static audit finding; see `AUDIT_REPORT.md` P2-17).
2. Observe: `Stashing` list ignores `stash_untracked` & `keep_index` toggles

## Fix Log & Agent Notes
