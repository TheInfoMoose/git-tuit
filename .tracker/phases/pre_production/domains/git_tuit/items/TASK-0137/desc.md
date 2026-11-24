## Problem Description
Since 13B a theme.ron without base fills unset fields from classic instead of amber (src/ui/style.rs:614, D22). A truecolor user with a Phase 12 partial theme.ron silently flips from amber to classic; nothing in the log or UI says base: Some("amber") restores it. Product call: log/changelog note or nothing.
Found during: Phase 13 review.

## Reproduction Steps
COLORTERM=truecolor, theme.ron = (selection_bg: Some("Black")) -> classic colors, no log line mentioning base.

## Fix Log & Agent Notes
