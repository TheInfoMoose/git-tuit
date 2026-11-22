## Problem Description
`src/popups/confirm.rs` (`get_text`), `src/popups/reset.rs:30-53`, `src/popups/checkout_option.rs` — parts of P2-06 and P2-07 that no open item carries: hard reset and "discard all" have no second confirmation (BUG-0040's close note moved this to P2-08, but BUG-0042 doesn't mention it); Up on the reset picker wraps from Soft to Hard; the stash-pop, remote-tag delete, hunk/line discard, force-push and undo prompts don't name their stash, tag, path, remote or commit. — suggested fix: confirm the two destructive picks, stop the wrap, and name each target.
Kind: bug
Found during: Phase 8 planning (Claude, code trace, not executed).

## Reproduction Steps
1. Open the reset popup, press Up, then Enter.
2. Observe: a hard reset runs with no second prompt.

## Fix Log & Agent Notes
