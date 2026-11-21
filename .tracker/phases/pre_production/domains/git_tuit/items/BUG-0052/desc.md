## Problem Description
`src/queue.rs:23-35`, `tabs/status.rs:384-899`, `popups/branchlist.rs:321-342,506`, `submodules.rs:364` — `NeedsUpdate::REMOTES = 0b1001` collision; `Status` remote/detached-HEAD bugs; `BranchList` fetch/HEAD/truncate bugs — suggested fix: Set `REMOTES = 0b10000`; call `check_remotes()`; fix `can_fetch`/`can_push`; refresh branches on `Fetch`; assign `.unicode_truncate()`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-18`). Planned phase: 6.

## Reproduction Steps
1. Inspect `src/queue.rs:23-35`, `tabs/status.rs:384-899`, `popups/branchlist.rs:321-342,506`, `submodules.rs:364` (static audit finding; see `AUDIT_REPORT.md` P2-18).
2. Observe: `NeedsUpdate::REMOTES = 0b1001` collision; `Status` remote/detached-HEAD bugs; `BranchList` fetch/HEAD/truncate bugs

## Fix Log & Agent Notes
