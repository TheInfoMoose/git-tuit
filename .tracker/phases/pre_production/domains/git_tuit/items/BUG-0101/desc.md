## Problem Description
load_with_notice in src/ui/style.rs tries the patch format (colors in Some) then the old format (bare colors). A file mixing both fails both parses and falls back to plain Theme::classic() with only an error log, ignoring base too. THEMES.md (15A wording) says a file that sets base "may write every color without Some", which reads as per-field permission. Fix options: document all-or-nothing explicitly, or accept either form per field.
Found during: Phase 15 review.

## Reproduction Steps
theme.ron: (base: Some("amber"), tag_fg: Some("Red"), canvas_bg: "Black") -> gitui loads the classic theme, not amber; only visible with --logging.

## Fix Log & Agent Notes
