## Problem Description
src/popups/checkout_option.rs:864 asserts only res.is_err(). If an earlier check starts failing, the test still passes and the no-leftover-branch assertion becomes trivially true, so the branch rollback stops being tested.
Found during: Phase 9 review.

## Reproduction Steps
Make resolve_discard_target fail for this setup -> test still passes while the set_upstream rollback is never reached.

## Fix Log & Agent Notes
