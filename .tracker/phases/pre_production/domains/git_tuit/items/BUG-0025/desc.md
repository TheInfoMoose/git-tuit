## Problem Description
`src/popups/externaleditor.rs:69-78,112-116` — Untrusted local `core.editor` RCE & CWD binary hijack — suggested fix: Use `get_global_config_string("core.editor")`, reject relative sep/`..`, resolve sanitized `PATH`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P1-13`). Planned phase: 4.

Parked at Phase 3 review (git-parity threat model): git runs repo-local core.editor too (git parity).

## Reproduction Steps
1. Inspect `src/popups/externaleditor.rs:69-78,112-116` (static audit finding; see `AUDIT_REPORT.md` P1-13).
2. Observe: Untrusted local `core.editor` RCE & CWD binary hijack

## Fix Log & Agent Notes
