## Problem Description
`src/components/commitlist.rs:239-297,735-768` — since 961636ad (STEP 7N), search results that land during a history reload don't resolve `pending_restore`.

(a) Defect, if the selected match should win (assumed product decision, used as the replay's answer key): `set_highlighting` -> `select_next_highlight` moves the selection to a match but leaves the restore pending. When the saved commit loads (or loading ends), the restore moves the selection off the match.
- A match had loaded when the search landed: regression from 7N.
- The list was still empty: older than 7N. The old code didn't land on a match afterwards either.
- Cause: a gap in the 7N spec (written by Claude). It didn't define precedence between restore and search landing. Case 1e names only a user move and jump-to-commit, and "Must still work: search highlighting" is generic.
- Not attributed to the coder or validator. In a validator-only replay (2026-11-21, 3 runs per cell) the unchanged rules caught it 0/3 on the original spec and 3/3 with one added spec line stating the precedence. The coder wasn't tested.

(b) Open decision, not a confirmed defect: does a movement key that can't move cancel a pending restore?
- Highlight mode: no (`:388-390`).
- Normal mode: no while the list is empty (`:409-411`), yes once commits have loaded (`:413`).
- Case 1e covers "moves the selection" and doesn't settle this.

Test gaps: `t7n_reload_user_choice_wins` has no highlights. `t7n_reload_missing_commit_clamps` doesn't call `finish_loading` on an empty list.
— suggested fix: state the precedence for search landing (including on an empty list) and for (b) in the step, then add one test each.
Kind: bug
Found during: STEP 7N phase review (Claude, read-only; not reproduced).
Corrected 2026-11-21 after the pilot root-cause check and replay: (b) is no longer claimed as a 1e violation, the regression claim is narrowed, and the cause is the spec gap.

## Reproduction Steps
From a code trace, not executed.
(a)
1. In a repo with well over 3000 commits, select an old commit in the Log tab and start a search.
2. Before the search ends, change HEAD from another terminal so the history reloads.
3. Let the search finish after a match has loaded but before the old commit has. Observe: the selection ends on the old commit, not on a match.

(b)
1. With search results shown and an old match selected, change HEAD.
2. Press Up before the old match loads. Observe: nothing moves, and the old match is selected again when it loads.

## Fix Log & Agent Notes
