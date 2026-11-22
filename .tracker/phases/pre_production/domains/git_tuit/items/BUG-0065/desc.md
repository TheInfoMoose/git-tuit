## Problem Description
`src/components/diff.rs:266-268` — 8e1f07f6 (STEP 7J) item 3 is incomplete: `is_empty()` is `!pending && diff.is_none()`, so a loaded diff with zero lines keeps focus. The test only covers no file selected — suggested fix: `!pending && lines_count() == 0`, with a loaded zero-line test.
Kind: bug
Found during: Phase 7 pilot review (Claude, read-only; not reproduced)

## Reproduction Steps
1. Select an empty new file in the Status tab and focus its diff.
2. Observe: focus stays in the empty diff.

## Fix Log & Agent Notes
