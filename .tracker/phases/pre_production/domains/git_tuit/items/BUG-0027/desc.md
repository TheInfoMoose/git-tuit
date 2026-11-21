## Problem Description
`src/popups/externaleditor.rs:80-108` — Broken quote/whitespace splitting for editor commands — suggested fix: Implement `parse_editor_command` tokenizer handling quotes/escapes cleanly + unit tests
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-15`). Planned phase: 4.

## Reproduction Steps
1. Inspect `src/popups/externaleditor.rs:80-108` (static audit finding; see `AUDIT_REPORT.md` P1-15).
2. Observe: Broken quote/whitespace splitting for editor commands

## Fix Log & Agent Notes
