## Problem Description
The 14B info! line (src/ui/style.rs:684-689) is asserted by no test; deleting it passes everything. The plan skipped a capture logger because test_smoke installs env_logger. Also, logging only exists with --logging (src/args.rs:49,148), so most users never see the hint. Options: route the message through a pure fn that returns the notice and test that, and/or surface it in the UI once.
Found during: Phase 14 review.

## Reproduction Steps
Delete the log::info! in the base None arm of init_for; cargo test --workspace still passes.

## Fix Log & Agent Notes
