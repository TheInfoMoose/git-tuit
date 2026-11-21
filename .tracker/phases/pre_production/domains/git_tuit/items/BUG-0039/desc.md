## Problem Description
`filetreelist/src/filetreeitems.rs:218,258`, `commitlist.rs:277,711`, `statustree.rs:231-258`, `status_tree.rs:178`, `filetree.rs:213`, `taglist.rs:391`, `blame_file.rs:329,706,788`, `branchlist.rs:590`, `confirm.rs:194`, `remotelist.rs:416-419` — Production underflow, OOB indexing & `.expect()` panics — suggested fix: Replace `- 1` with `checked_sub`/`saturating_sub`, bounds-check `.get()`, remove `.expect()`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-24`). Planned phase: 5.

## Reproduction Steps
1. Inspect `filetreelist/src/filetreeitems.rs:218,258`, `commitlist.rs:277,711`, `statustree.rs:231-258`, `status_tree.rs:178`, `filetree.rs:213`, `taglist.rs:391`, `blame_file.rs:329,706,788`, `branchlist.rs:590`, `confirm.rs:194`, `remotelist.rs:416-419` (static audit finding; see `AUDIT_REPORT.md` P1-24).
2. Observe: Production underflow, OOB indexing & `.expect()` panics

## Fix Log & Agent Notes
