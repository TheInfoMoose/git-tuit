## Problem Description
Theme::tags(false) (src/ui/style.rs) sets bg Color::Reset; used at src/components/commitlist.rs:553. Since 15D paints the frame with canvas_bg (#000000 on amber), unselected tag labels in the log show the terminal default background as a hole in the black canvas. Before 15D the whole screen was Reset so it was invisible. 15D deferred explicit-Reset widgets to the pane migration (TASK-0008); this is the concrete instance. Sibling to check in the same pass: popups that render Clear (resets cells to Reset) without repainting panel_bg/canvas_bg over the full area.
Found during: Phase 15 review.

## Reproduction Steps
Amber theme (no theme.ron, COLORTERM=truecolor) on a terminal whose default bg is not #000000 (e.g. a light theme); open a repo with a tagged commit; Log tab: the tag label cells show the terminal bg instead of black.

## Fix Log & Agent Notes
