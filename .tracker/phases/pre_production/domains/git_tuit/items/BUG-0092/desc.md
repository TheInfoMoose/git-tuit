## Problem Description
Phase 12 changed every Theme::default() value to amber RGB and added four fields. Patch-format theme.ron files (hand-written, or produced by the launch save_patch conversion, which stored only fields that differed from the old ANSI defaults) now resolve every unset field to amber, so a user theme silently changes on upgrade. Old full-format files also get amber panel_bg, border_focused_fg, scroll_track_fg and attention_fg. This breaks ~/.config/gitui/ compatibility. Review F2, .phased/reviews/12.md.
Found during: Phase 12 review.

## Reproduction Steps
Write theme.ron as (selection_bg: Some("LightBlue")) and launch: tab, titles, borders, hashes, branches all render amber instead of the launch ANSI colors. src/ui/style.rs Theme::init and Default impl.

## Fix Log & Agent Notes
