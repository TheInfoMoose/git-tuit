## Problem Description
draw_scrollbar styles the position cell with scroll_bar_pos(), which is selection_bg (#2A1A00 by default). The new track is scroll_track_fg (#5C3A00), so the thumb is darker than the track and almost black on a dark terminal, and the scroll position is hard to see. t12b_scrollbar_track_styled asserts the thumb at Rgb(42,26,0), so the test locks this in. Review F3, .phased/reviews/12.md.
Found during: Phase 12 review.

## Reproduction Steps
Open any list or diff longer than its pane with the default theme: the track shows dark amber and the position marker is close to invisible. src/ui/scrollbar.rs:140-141, src/ui/style.rs scroll_bar_pos.

## Fix Log & Agent Notes
