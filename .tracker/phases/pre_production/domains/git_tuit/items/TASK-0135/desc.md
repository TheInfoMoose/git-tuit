## Problem Description
STEP 13B spec gap: STEP 13B changes Theme::init so patch files without a `base` key fill unset fields from Theme::classic() (scroll_track_fg: Color::Reset, scroll_thumb_fg follows selection_bg), and requires `cargo test --workspace --locked` in POST_GATE, but ALLOWED_FILES only permits `src/ui/style.rs` and `THEMES.md`. `t13a_thumb_color_from_patch` in `src/ui/scrollbar.rs:209-234` loads two patch files without `base` (`(scroll_thumb_fg: Some("Cyan"))` and `(selection_bg: Some("Black"))`) and asserts amber fills (`Color::Rgb(92, 58, 0)` track and `Color::Rgb(255, 216, 102)` thumb), so `cargo test --workspace --locked` cannot pass without modifying `src/ui/scrollbar.rs` (either updating assertions to classic/thumb-follow values or adding `base: Some("amber")` to the test fixtures). Location: `src/ui/scrollbar.rs:209-234`.
Lapse: L2 file outside ALLOWED_FILES
Kind: scope-lapse
Found during: STEP 13B (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
