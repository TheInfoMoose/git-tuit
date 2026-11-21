## Problem Description
`src/popups/{push.rs:315-326,push_tags.rs:237-247,pull.rs:70-87,249-278,fetch.rs:62-80,162-191}` — `Esc` on cred prompt runs Push/Pull/Fetch anyway; `Pull`/`Fetch` modal lockup on preflight error — suggested fix: Abort & hide on cred cancel (`!is_complete()`); handle `Esc`/`Enter` & hide on error in `Pull`/`Fetch`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-10`). Planned phase: 5.

## Reproduction Steps
1. Inspect `src/popups/{push.rs:315-326,push_tags.rs:237-247,pull.rs:70-87,249-278,fetch.rs:62-80,162-191}` (static audit finding; see `AUDIT_REPORT.md` P2-10).
2. Observe: `Esc` on cred prompt runs Push/Pull/Fetch anyway; `Pull`/`Fetch` modal lockup on preflight error

## Fix Log & Agent Notes
