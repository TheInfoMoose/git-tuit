## Problem Description
`.github/workflows/ci.yml:158-355` — 5 upstream jobs (`arm`, `apple`, `udeps`, `log-test`, `test-homebrew`) — suggested fix: Delete all 5 upstream-only jobs
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-05`). Planned phase: 2.

## Reproduction Steps
1. Inspect `.github/workflows/ci.yml:158-355` (static audit finding; see `AUDIT_REPORT.md` P0-05).
2. Observe: 5 upstream jobs (`arm`, `apple`, `udeps`, `log-test`, `test-homebrew`)

## Fix Log & Agent Notes
