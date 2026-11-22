## Problem Description
`src/tabs/status.rs:320-326` — `can_focus_diff()` only checks that a file is selected. Since 961636ad (STEP 7N) a loaded zero-line diff returns focus at once, as specified, so the focus-diff hint is shown enabled for empty, mode-only and other zero-line diffs, and the key does nothing — suggested fix: disable the hint and key once the diff has loaded with zero lines; keep both while it loads.
Cause: gap in the 7N spec (written by Claude), not a coder or validator miss.
Kind: bug
Found during: STEP 7N phase review (Claude, read-only; not reproduced).
Corrected 2026-11-21 after the pilot root-cause check: line reference (was 329-335) and cause.

## Reproduction Steps
From a code trace, not executed.
1. Select an empty new file in the Status tab.
2. Observe: the focus-diff hint is enabled, and Right leaves focus in the file list.

## Fix Log & Agent Notes
