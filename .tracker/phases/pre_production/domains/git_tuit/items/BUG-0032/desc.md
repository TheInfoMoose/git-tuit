## Problem Description
`src/components/cred.rs:63-161` — `CredComponent::hide()` retains password & causes blind input hijack — suggested fix: Hide & `.clear()` both `input_username` & `input_password` on hide/submit
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-20`). Planned phase: 4.

## Reproduction Steps
1. Inspect `src/components/cred.rs:63-161` (static audit finding; see `AUDIT_REPORT.md` P1-20).
2. Observe: `CredComponent::hide()` retains password & causes blind input hijack

## Fix Log & Agent Notes
