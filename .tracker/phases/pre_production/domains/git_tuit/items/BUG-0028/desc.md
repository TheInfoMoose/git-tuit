## Problem Description
`src/popups/externaleditor.rs:64-67` — `.expect("reset terminal")` inside `defer!` guard — suggested fix: Log error instead of panicking inside drop guard
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-16`). Planned phase: 4.

## Reproduction Steps
1. Inspect `src/popups/externaleditor.rs:64-67` (static audit finding; see `AUDIT_REPORT.md` P1-16).
2. Observe: `.expect("reset terminal")` inside `defer!` guard

## Fix Log & Agent Notes
