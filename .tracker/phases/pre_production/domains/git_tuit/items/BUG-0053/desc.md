## Problem Description
`vim_style_key_config.ron:18-19,29`, `textinput.rs:368-396` — Vim `Shift+U` collision; singleline `TextInput` eats `Ctrl+n`/`Ctrl+p` — suggested fix: Gate vertical cursor moves to `Multiline`; resolve vim `Shift+U` mapping cleanly
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-19`). Planned phase: 6.

## Reproduction Steps
1. Inspect `vim_style_key_config.ron:18-19,29`, `textinput.rs:368-396` (static audit finding; see `AUDIT_REPORT.md` P2-19).
2. Observe: Vim `Shift+U` collision; singleline `TextInput` eats `Ctrl+n`/`Ctrl+p`

## Fix Log & Agent Notes
