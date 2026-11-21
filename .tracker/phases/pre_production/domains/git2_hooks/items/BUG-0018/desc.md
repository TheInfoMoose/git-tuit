## Problem Description
`git2-hooks/src/hookspath.rs:42-133,296-310` — `shellexpand::full` `$VAR` expansion, hook traversal, dir/world-writable exec — suggested fix: Use `shellexpand::tilde`, validate `Component::Normal` hook, canonicalize relative `hooksPath`, require file & `!world_writable`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-05`). Planned phase: 4.

## Reproduction Steps
1. Inspect `git2-hooks/src/hookspath.rs:42-133,296-310` (static audit finding; see `AUDIT_REPORT.md` P1-05).
2. Observe: `shellexpand::full` `$VAR` expansion, hook traversal, dir/world-writable exec

## Fix Log & Agent Notes
