## Problem Description
`.orrery/skills/repo-rules.md` ("Outcome steps", Validator) — proposed after the 7N review: make the validator trace every writer of changed state and report evidence rows. Tested and not supported. Recommendation: close as won't fix (the user's call at triage).

Validator-only replay, 2026-11-21, 3 runs per cell, scored by a fresh session without the key:

| Arm | 7N defect caught | False denials on 7K | Tokens on 7N vs. A |
|---|---|---|---|
| A: current rules | 0/3 | 0/3 | - |
| B: A + trace every writer | 0/3 | 0/3 | +7% |
| C: B + evidence rows | 0/3 | 0/3 | +17% |
| D: A + one spec line stating the precedence | 3/3 | not run | -1% |

- The diagnosis this item was filed on is withdrawn: validator coverage and reporting didn't cause the 7N miss. B and C validators displayed the search writer and still confirmed, which the spec as written allowed.
- Still true, but not shown to affect quality: the saved report carries only `summary` and `feedback`, so traces in an agent's final message don't reach the next stage.

— suggested fix: none here. The planning-side rule is tracked in TASK-0091.
Kind: process
Found during: STEP 7N phase review (Claude, read-only).
Corrected 2026-11-21 with the replay result.

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
