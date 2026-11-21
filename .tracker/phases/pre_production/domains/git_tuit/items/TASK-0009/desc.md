## Problem Description
`src/popups/{msg,externaleditor,goto_line,fuzzy_find,log_search,help,options,remotelist,submodules,branchlist,...}.rs` — `MsgPopup` info red title, `ExternalEditor` body color hazard, unstyled popups — suggested fix: Apply `Clear` + `PANEL_BG` + `ui::popup_block`, fix `MsgPopup::show_info`, `ExternalEditor`, `FuzzyFind`/`LogSearch` `.style()`, remove raw `Color::Red`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P3-07`). Planned phase: 7.

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
