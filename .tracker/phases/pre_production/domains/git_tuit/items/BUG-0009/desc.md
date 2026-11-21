## Problem Description
`.github/workflows/cd.yml:1-134` — Upstream multi-OS/Homebrew tag release workflow — suggested fix: Trim to Linux-only `ubuntu-latest` (with `gpgsm`/`musl-tools`) or disable
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-09`). Planned phase: 2.

## Reproduction Steps
1. Inspect `.github/workflows/cd.yml:1-134` (static audit finding; see `AUDIT_REPORT.md` P0-09).
2. Observe: Upstream multi-OS/Homebrew tag release workflow

## Fix Log & Agent Notes
