## Problem Description
`STEPPED_IMPLEMENTATION_PLAN.md:147,150,154` — the IMAGE user step holds absolute host paths, and they are in local commits not yet on origin. The untracked `docs/baseline/clippy.txt` and `docs/baseline/test.txt` hold host paths too — suggested fix: before the first push, decide whether to scrub the unpushed history or accept it; keep the baseline logs untracked or redact them.
Kind: hygiene
Found during: STEP 7N phase review (Claude, read-only).

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
