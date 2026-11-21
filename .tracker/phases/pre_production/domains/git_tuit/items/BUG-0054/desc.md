## Problem Description
`utils/statustree.rs:55-70,192-207`, `commitlist.rs:93-557`, `utils/mod.rs:49-69` — `StatusTree` binary-searches unsorted items; `CommitList` OOB/mark-slice/byte-width bugs — suggested fix: Linear lookup in `StatusTree`; clamp `CommitList` selection; absolute mark index; `UnicodeWidthStr` widths
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-20`). Planned phase: 6.

## Reproduction Steps
1. Inspect `utils/statustree.rs:55-70,192-207`, `commitlist.rs:93-557`, `utils/mod.rs:49-69` (static audit finding; see `AUDIT_REPORT.md` P2-20).
2. Observe: `StatusTree` binary-searches unsorted items; `CommitList` OOB/mark-slice/byte-width bugs

## Fix Log & Agent Notes
