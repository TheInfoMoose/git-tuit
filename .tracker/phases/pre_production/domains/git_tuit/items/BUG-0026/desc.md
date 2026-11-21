## Problem Description
`src/popups/externaleditor.rs:48-62,107-116` — External editor path traversal, symlink escape & missing `--` — suggested fix: Canonicalize target inside `work_dir` or `git_dir`, check `is_file()`, pass `--` before path
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-14`). Planned phase: 4.

Parked at Phase 3 review (git-parity threat model): the target path is gitui-chosen and absolute; git opens the editor the same way.

## Reproduction Steps
1. Inspect `src/popups/externaleditor.rs:48-62,107-116` (static audit finding; see `AUDIT_REPORT.md` P1-14).
2. Observe: External editor path traversal, symlink escape & missing `--`

## Fix Log & Agent Notes
