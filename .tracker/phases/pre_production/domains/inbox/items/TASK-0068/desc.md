## Problem Description
Imported from roadmap step `STEP 6J`: Blame errors stop the spinner
- Step ID: `STEP 6J`
- Command: `/goal`
- Roadmap order: 6
- Target files:
  - `asyncgit/src/blame.rs`
  - `src/popups/blame_file.rs`

Partly advances (tag/remote-URL part moves to Phase 7): BUG-0055 (P2-21)

## Reproduction Steps
1. Inspect `.tracker`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.tracker`
