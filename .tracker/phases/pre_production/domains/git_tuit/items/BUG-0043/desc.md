## Problem Description
`src/app.rs:316-368,495-572` — Popup Z-order vs event-capture inversion & unblocked app shortcuts — suggested fix: Enforce strict reverse order in `accessors!`/`setup_popups!`; guard tab/options shortcuts with `!any_popup_visible()`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-09`). Planned phase: 6.

## Reproduction Steps
1. Inspect `src/app.rs:316-368,495-572` (static audit finding; see `AUDIT_REPORT.md` P2-09).
2. Observe: Popup Z-order vs event-capture inversion & unblocked app shortcuts

## Fix Log & Agent Notes
