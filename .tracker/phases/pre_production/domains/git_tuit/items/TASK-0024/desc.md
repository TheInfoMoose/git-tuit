## Problem Description
`deny.toml:28,39` — `{ crate = "windows-sys" }` and `{ crate = "windows-core" }` match nothing since STEP 2D's Linux-only `[graph] targets`, so cargo-deny prints `warning[unmatched-skip-root]` on every run — suggested fix: delete both entries and their comment lines.
Found during: Phase 2 review (STEP 2D/2H gate logs).

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
