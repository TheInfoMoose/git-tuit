## Problem Description
A theme.ron with a base key plus a bare (non-Some) color, e.g. (base: "amber", selection_bg: "Black"), is rejected whole by the 14A guard (src/ui/style.rs:711-734): plain classic theme, every override dropped, only an error! that needs --logging. At launch the same file loaded as classic + its colors (then was rewritten, dropping base). THEMES.md now tells users base takes a bare string, which invites bare colors too. Non-destructive is right; silently losing all colors is a regression. Product call: apply the old-format colors on the named base in memory and skip the rewrite, or keep full rejection.
Found during: Phase 14 review.

## Reproduction Steps
Write (base: "amber", selection_bg: "Black") to theme.ron, start gitui: classic colors, selection_bg not Black. t14a_base_file_never_rewritten case 1 asserts this.

## Fix Log & Agent Notes
