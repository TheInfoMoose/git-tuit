## Problem Description
ThemePatchFile.base is Option<String>; a bare base: "amber" fails the patch parse, OldThemeFile ignores the unknown base key and parses, so init treats the file as old full-format, loads classic and save_patch overwrites the user file (src/ui/style.rs:62, 627). At launch the same file parsed as an empty patch and was never rewritten.
Found during: Phase 13 review.

## Reproduction Steps
theme.ron containing (base: "amber") -> gitui starts with classic colors and theme.ron is rewritten to (). Same for (base: "amber", selection_bg: "Black").

## Fix Log & Agent Notes
