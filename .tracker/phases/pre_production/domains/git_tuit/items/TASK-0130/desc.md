## Problem Description
assert_theme_eq and t12a_default_is_amber in src/ui/style.rs check only the 24 original fields. border_focused_fg, scroll_track_fg, panel_bg and attention_fg are skipped, so t12a_partial_old_theme_loads and t12a_bad_file_gives_default would still pass if old-format conversion dropped or mangled them. Review F5, .phased/reviews/12.md.
Found during: Phase 12 review.

## Reproduction Steps
Break the default for scroll_track_fg only on the old-format conversion path: the t12a tests still pass. src/ui/style.rs mod tests, assert_theme_eq.

## Fix Log & Agent Notes
