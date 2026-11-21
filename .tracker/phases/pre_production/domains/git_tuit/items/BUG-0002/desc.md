## Problem Description
`.github/workflows/ci.yml:15-100` — Multi-OS/nightly build matrix & OS steps — suggested fix: Restrict `build` to `ubuntu-latest` (`[stable, "1.88"]`), remove macOS/Windows steps
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-02`). Planned phase: 2.

## Reproduction Steps
1. Inspect `.github/workflows/ci.yml:15-100` (static audit finding; see `AUDIT_REPORT.md` P0-02).
2. Observe: Multi-OS/nightly build matrix & OS steps

## Fix Log & Agent Notes
