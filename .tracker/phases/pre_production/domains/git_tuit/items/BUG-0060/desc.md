## Problem Description
`Cargo.lock` (heapless 0.8.0 via faster-hex 0.10.1 → gix-hash 0.25.1) — heapless 0.8.0 is affected by RUSTSEC-2026-0574..0582 (unsound, patched >= 0.9.4) and is built for x86_64-unknown-linux-gnu, but `cargo deny check` passes (`.git/phased/steps/2H/post_gate.log:86`); likely cargo-deny's default `unsound` scope only fails workspace crates (guess) — suggested fix: set `unsound = "all"` under `[advisories]` in `deny.toml`, then resolve heapless via a gix/faster-hex update or a reviewed ignore.
Found during: Phase 2 review (STEP 2H). Follow-up to TASK-0004 (P0-12).

## Reproduction Steps
1. `cargo tree -i heapless --target x86_64-unknown-linux-gnu -e normal` → heapless v0.8.0 ← faster-hex v0.10.1
2. `cargo deny check advisories` → `advisories ok`

## Fix Log & Agent Notes
