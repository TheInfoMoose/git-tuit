## Problem Description
`commit_details/{details.rs:184-266,mod.rs:147-168}`, `utils/{scroll_vertical.rs,scroll_horizontal.rs}`, `syntax_text.rs:98-137,218-222`, `strings.rs:426-439` — `Details` collapses to 0 rows at 80x24 & clips tags; scroll OOB on resize; `PageUp`/`Down` inverted at 0 height; `ellipsis_trim_start` overflow — suggested fix: Proportional detail height, clamp scroll on resize, guard `visual_height == 0`, reset `SyntaxText` scroll on load, guard `width < 3`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-16`). Planned phase: 6.

## Reproduction Steps
1. Inspect `commit_details/{details.rs:184-266,mod.rs:147-168}`, `utils/{scroll_vertical.rs,scroll_horizontal.rs}`, `syntax_text.rs:98-137,218-222`, `strings.rs:426-439` (static audit finding; see `AUDIT_REPORT.md` P2-16).
2. Observe: `Details` collapses to 0 rows at 80x24 & clips tags; scroll OOB on resize; `PageUp`/`Down` inverted at 0 height; `ellipsis_trim_start` overflow

## Fix Log & Agent Notes
