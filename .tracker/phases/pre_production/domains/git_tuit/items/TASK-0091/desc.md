## Problem Description
`CLAUDE.md` ("Planning outcome steps") — outcome steps written by Claude left out cases, so the validator had no rule to check. Two kinds were missed: what wins between writers of the same state (BUG-0067(a)), and an input state covered by a condition word (BUG-0065, "no content" with a loaded zero-line diff). — suggested fix: when planning an outcome step:
1. For each piece of state the step changes, list every writer, changed or not. Give each an edge case saying what wins.
2. For each condition the step names (empty, loaded, missing), give an edge case for each state it covers.
3. If either answer is a product call, ask the user before writing the step.

Evidence (confirmation replay, 2026-11-21, validator only, 10 fresh runs per cell, criteria frozen beforehand, denials scored by a fresh session):

| Case | Current rules | Current rules + one spec line |
|---|---|---|
| 7N, BUG-0067(a), caught | 0/10 | 10/10 |
| 7J holdout, BUG-0065, caught | 0/10 | 10/10 |
| 7K control, false denials | not run | 0/10 |

Result: confirmed by the frozen criteria. Every cell was unanimous.

State: adopted by the user on 2026-11-21 for a one-phase trial in this repo only.
Done when: one phase has been planned with the rule and its review has counted spec gaps, and the user has decided whether to keep it and whether it goes into the global phased-workflow skill. 7O doesn't count as the trial, because its answers are already known.
Not shown by the replay: that a planner finds these cases, or that the coder follows them.
Kind: process
Found during: STEP 7N validator replay (Claude).

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
