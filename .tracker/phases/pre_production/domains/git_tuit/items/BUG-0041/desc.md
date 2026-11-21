## Problem Description
`src/popups/confirm.rs:131-216`, `src/strings.rs:146-239`, `src/components/diff.rs:547-591,915-942` — Missing targets/untracked warnings in `ConfirmPopup`; `0 selected lines` prompt; `ForcePush` truncates branch — suggested fix: Name target paths/branches/remotes/tags/hashes cleanly; warn on untracked deletion; guard empty `selected_lines`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-07`). Planned phase: 5.

## Reproduction Steps
1. Inspect `src/popups/confirm.rs:131-216`, `src/strings.rs:146-239`, `src/components/diff.rs:547-591,915-942` (static audit finding; see `AUDIT_REPORT.md` P2-07).
2. Observe: Missing targets/untracked warnings in `ConfirmPopup`; `0 selected lines` prompt; `ForcePush` truncates branch

## Fix Log & Agent Notes
