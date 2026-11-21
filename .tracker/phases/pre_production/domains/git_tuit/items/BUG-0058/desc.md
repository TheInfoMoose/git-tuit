## Problem Description
`src/cmdbar.rs:37-198`, `src/app.rs:351-356,1160-1228`, `popups/{confirm.rs:38,reset.rs:160,checkout_option.rs:130}` — `CommandBar` blank on narrow width, hardcoded `"more [.]"`, missing `r.x +`, popup `.` swallow, cramped confirm rects — suggested fix: Guard `line_width > 0` before `LineBreak`, dynamic toggle hint, include `r.x +`, cap height, allow non-input popup `.`, responsive confirm rects
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-24`). Planned phase: 7.

## Reproduction Steps
1. Inspect `src/cmdbar.rs:37-198`, `src/app.rs:351-356,1160-1228`, `popups/{confirm.rs:38,reset.rs:160,checkout_option.rs:130}` (static audit finding; see `AUDIT_REPORT.md` P2-24).
2. Observe: `CommandBar` blank on narrow width, hardcoded `"more [.]"`, missing `r.x +`, popup `.` swallow, cramped confirm rects

## Fix Log & Agent Notes
