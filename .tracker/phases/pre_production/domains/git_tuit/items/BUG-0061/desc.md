## Problem Description
`src/args.rs` `open_log_file` (STEP 4I, 08f7022b) — `OpenOptions::mode(0o600)` only applies when the file is created; a `gitui.log` left by an earlier run keeps its old mode (typically 0644), and `truncate(true)` doesn't change it, so the trace log stays world-readable for existing users — suggested fix: after opening, on unix call `file.set_permissions(fs::Permissions::from_mode(0o600))?`, and add a test that pre-creates the log with 0644 and asserts 0600 after `open_log_file`.
Found during: Phase 4 review (STEP 4I).

## Reproduction Steps
1. `install -m 644 /dev/null ~/.cache/gitui/gitui.log`
2. `gitui -l`, quit, then `stat -c %a ~/.cache/gitui/gitui.log` → `644`

## Fix Log & Agent Notes
