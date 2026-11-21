## Problem Description
`revision_files.rs:133,260-530`, `tabs/{files.rs:36,status.rs:467-480,revlog.rs:464-474}`, `app.rs:182-296,634`, `diff.rs:276`, `keys/{key_list.rs,symbols.rs,key_config.rs}`, `textinput.rs`, `status_tree.rs:147`, `strings.rs:693,759,1328,1772` — Unborn-HEAD hang, invalid persisted tab crash, focus/nav polish, key-parse logging, `Tab` in singleline, hint typos — suggested fix: Clear loading on unborn HEAD/error, clamp persisted tab, fix `Focus::Diff`/`Revlog` `Esc`/key hints, log key-config errors
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-22`). Planned phase: 6.

## Reproduction Steps
1. Inspect `revision_files.rs:133,260-530`, `tabs/{files.rs:36,status.rs:467-480,revlog.rs:464-474}`, `app.rs:182-296,634`, `diff.rs:276`, `keys/{key_list.rs,symbols.rs,key_config.rs}`, `textinput.rs`, `status_tree.rs:147`, `strings.rs:693,759,1328,1772` (static audit finding; see `AUDIT_REPORT.md` P2-22).
2. Observe: Unborn-HEAD hang, invalid persisted tab crash, focus/nav polish, key-parse logging, `Tab` in singleline, hint typos

## Fix Log & Agent Notes
