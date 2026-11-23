## Problem Description
In Phase 12 the agy-hydrogen-val stage skipped cargo clippy and cargo test for STEP 12E, saying the /workspace mount was read-only, and still wrote Confirmed. The validator stage is not actually checking the gate.
Found during: Phase 12 review.

## Reproduction Steps
phase-role log for STEP 12E, 2026-11-23T15:19:18: clippy/test skipped on read-only mount, deferred to host gate.

## Fix Log & Agent Notes
