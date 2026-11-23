## Problem Description
Theme::default() is now all Color::Rgb with no truecolor or background detection. The launch defaults were ANSI named colors and Reset, which follow the terminal palette. On a light background, #FFD866 (focused titles, selected tab, hashes, tags) is about 1.3:1 contrast. On terminals without 24-bit color (Apple Terminal, tmux without Tc, Linux console) colors get approximated or mangled. README.md:267 and THEMES.md:3-4 still show assets/light-theme.png as the default. Review F1, .phased/reviews/12.md.
Found during: Phase 12 review.

## Reproduction Steps
Run with no theme.ron on a white-background terminal or with TERM=xterm-256color and no truecolor: focused block titles and the selected tab are unreadable. src/ui/style.rs:12-19 and the Default impl.

## Fix Log & Agent Notes
