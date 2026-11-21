## Problem Description
`src/popups/reset.rs:112-149`, `src/popups/checkout_option.rs:80-116` — Sticky `Hard`/`DiscardAllLocalChanges` defaults; no secondary confirm; `discard_status` runs before `checkout` — suggested fix: Reset defaults on `open()`, remove wrap-around, add `ConfirmPopup` for destructive choices, emit `NeedsUpdate::ALL`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-06`). Planned phase: 5.

## Reproduction Steps
1. Inspect `src/popups/reset.rs:112-149`, `src/popups/checkout_option.rs:80-116` (static audit finding; see `AUDIT_REPORT.md` P2-06).
2. Observe: Sticky `Hard`/`DiscardAllLocalChanges` defaults; no secondary confirm; `discard_status` runs before `checkout`

## Fix Log & Agent Notes
