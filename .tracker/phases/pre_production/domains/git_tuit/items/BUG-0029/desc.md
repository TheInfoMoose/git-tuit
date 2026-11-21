## Problem Description
`src/clipboard.rs:13-17,119` — `PATH` hijack & bare-command fallback execution — suggested fix: Resolve via `which_in` on absolute-only `PATH`, fail closed if absent, check non-empty `WAYLAND_DISPLAY`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-17`). Planned phase: 4.

## Reproduction Steps
1. Inspect `src/clipboard.rs:13-17,119` (static audit finding; see `AUDIT_REPORT.md` P1-17).
2. Observe: `PATH` hijack & bare-command fallback execution

## Fix Log & Agent Notes
