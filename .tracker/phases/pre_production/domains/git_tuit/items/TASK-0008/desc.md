## Problem Description
`components/{syntax_text.rs,diff.rs,commitlist.rs,revision_files.rs,commit_details/*,textinput.rs}`, `tabs/{status.rs,revlog.rs,stashing.rs}`, `ui/scrollbar.rs` — Swapped title/border (`syntax_text`), hardcoded focus/colors, unstyled scrollbar/badges — suggested fix: Migrate all primary panes, badges & scrollbar to `ui::styled_block` & `Theme` helpers
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P3-06`). Planned phase: 7.

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
