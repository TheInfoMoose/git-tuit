## Problem Description
`src/popups/fuzzy_find.rs:108-127` (refresh_selection) — the fuzzy finder pushes FuzzyFinderChanged on every highlight change, so the branch list / file tree already moved; Esc just hides the popup and leaves the last highlighted match selected instead of the selection the user had before opening it — suggested fix: let Enter commit and Esc cancel, e.g. send a FuzzyFinderChanged only on Enter, or remember the target's selection at open() and restore it on Esc. Remainder of audit P2-11.
Found during: Phase 6 review.

## Reproduction Steps
1. Open the branch list, press the fuzzy-find key, type a few letters so another branch highlights.
2. Press Esc → the branch list keeps the highlighted branch selected.

## Fix Log & Agent Notes
