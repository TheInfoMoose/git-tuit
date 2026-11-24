## Problem Description
Theme::init_for matches base exactly (src/ui/style.rs:610), so base: Some("Amber") is an unknown base and falls back to classic with only a log warning, while COLORTERM in the same function is matched case-insensitively.
Found during: Phase 13 review.

## Reproduction Steps
theme.ron = (base: Some("Amber")) -> classic colors, warn in log.

## Fix Log & Agent Notes
