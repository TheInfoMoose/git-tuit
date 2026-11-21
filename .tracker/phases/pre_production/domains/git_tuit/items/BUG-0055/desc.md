## Problem Description
`fetch.rs:105-114` (`fetch_job.rs`), `blame_file.rs:345-480` (`blame.rs`), `taglist.rs:348-353`, `remotelist.rs:460-471` — Silent fetch/blame/remote-tag/remote-URL failures & infinite blame spinner — suggested fix: Expose `AsyncFetchJob` & `AsyncBlame` errors (`STEP 3A`), show `ShowErrorMsg`, implement `BlameFilePopup::hide()`, handle `Ok(None)` URL
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-21`). Planned phase: 6.

## Reproduction Steps
1. Inspect `fetch.rs:105-114` (`fetch_job.rs`), `blame_file.rs:345-480` (`blame.rs`), `taglist.rs:348-353`, `remotelist.rs:460-471` (static audit finding; see `AUDIT_REPORT.md` P2-21).
2. Observe: Silent fetch/blame/remote-tag/remote-URL failures & infinite blame spinner

## Fix Log & Agent Notes
