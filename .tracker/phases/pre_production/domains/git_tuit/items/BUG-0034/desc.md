## Problem Description
`src/clipboard.rs:74-84,92,114` — Unbounded & unflushed OSC 52 payload — suggested fix: Cap payload (~75 KiB) & call `out.flush()?`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-03`). Planned phase: 4.

## Reproduction Steps
1. Inspect `src/clipboard.rs:74-84,92,114` (static audit finding; see `AUDIT_REPORT.md` P2-03).
2. Observe: Unbounded & unflushed OSC 52 payload

## Fix Log & Agent Notes
