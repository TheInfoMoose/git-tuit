## Problem Description
`.github/workflows/nightly.yml:1-125` — Upstream S3 nightly cron missing `gpgsm`/secrets — suggested fix: Delete `.github/workflows/nightly.yml`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-07`). Planned phase: 2.

## Reproduction Steps
1. Inspect `.github/workflows/nightly.yml:1-125` (static audit finding; see `AUDIT_REPORT.md` P0-07).
2. Observe: Upstream S3 nightly cron missing `gpgsm`/secrets

## Fix Log & Agent Notes
