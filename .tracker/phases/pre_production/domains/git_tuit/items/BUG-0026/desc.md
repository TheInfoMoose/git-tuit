## Problem Description
`src/popups/externaleditor.rs:48-62,107-116` — External editor path traversal, symlink escape & missing `--` — suggested fix: Canonicalize target inside `work_dir` or `git_dir`, check `is_file()`, pass `--` before path
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-14`). Planned phase: 4.

## Reproduction Steps
1. Inspect `src/popups/externaleditor.rs:48-62,107-116` (static audit finding; see `AUDIT_REPORT.md` P1-14).
2. Observe: External editor path traversal, symlink escape & missing `--`

## Fix Log & Agent Notes
