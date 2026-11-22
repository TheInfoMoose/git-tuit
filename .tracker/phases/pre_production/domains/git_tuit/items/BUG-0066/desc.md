## Problem Description
`src/popups/taglist.rs:344-362` — regression from 188e3826 (STEP 7L, caused by the spec): the background remote-tag lookup runs on every open and now raises a popup on any failure, so being offline, lacking stored credentials or having several remotes with no `origin` interrupts every open — suggested fix: show the failure inside the tag list (missing-remote state unknown) and keep the log line.
Kind: bug
Found during: Phase 7 pilot review (Claude, read-only; not reproduced)

## Reproduction Steps
1. In a repo with remote branches, go offline.
2. Open the tag list.
3. Observe: an error popup each time.

## Fix Log & Agent Notes
