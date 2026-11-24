## Problem Description
THEMES.md:10 opens with "To change the colors of the default theme you need to add a theme.ron", but adding one without base: Some("amber") replaces the amber default with classic. The rule follows in the same paragraph, after the misleading lead.
Found during: Phase 13 review.

## Reproduction Steps
Read THEMES.md Configuration section; follow only the first sentence on a truecolor terminal -> amber disappears.

## Fix Log & Agent Notes
