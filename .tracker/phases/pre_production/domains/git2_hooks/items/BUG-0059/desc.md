## Problem Description
`git2-hooks/src/lib.rs:699` — `test_pre_commit_py` writes a hook with shebang `#!/usr/bin/env python`; on hosts that only ship `python3` the hook exits 127 (`/usr/bin/env: 'python': No such file or directory`) and the test fails — suggested fix: use `#!/usr/bin/env python3` on unix (keep `python.exe` on Windows), or skip the test with a logged reason when no python interpreter is on PATH.
Found during: Phase 2 baseline (`docs/baseline/test.txt`). Planned phase: 3.

## Reproduction Steps
1. On a host without a `python` binary: `cargo test -p git2-hooks --lib tests::test_pre_commit_py`
2. Observe: `stderr: "/usr/bin/env: 'python': No such file or directory", code: 127`

## Fix Log & Agent Notes
