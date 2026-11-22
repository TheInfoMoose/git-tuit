## Problem Description
find_untracked_collisions (asyncgit/src/sync/branch/mod.rs:479-492) counts any un-indexed path that HEAD has as a collision, without comparing it to the HEAD blob. A git rm --cached file with unchanged content risks nothing but blocks the discard. Plan origin: 9B item 2.
Found during: Phase 9 review.

## Reproduction Steps
git rm --cached a.txt (content unchanged), discard-and-switch to any branch -> refused with an untracked-collision error naming a.txt.

## Fix Log & Agent Notes
