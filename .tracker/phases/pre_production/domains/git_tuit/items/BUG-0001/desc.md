## Problem Description
`.github/workflows/ci.yml:3-5` — Daily upstream cron schedule — suggested fix: Remove `schedule:` trigger; keep `push`/`pull_request` on `master`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-01`). Planned phase: 2.

## Reproduction Steps
1. Inspect `.github/workflows/ci.yml:3-5` (static audit finding; see `AUDIT_REPORT.md` P0-01).
2. Observe: Daily upstream cron schedule

## Fix Log & Agent Notes
