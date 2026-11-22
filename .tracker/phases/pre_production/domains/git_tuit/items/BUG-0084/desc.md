## Problem Description
`asyncgit/src/sync/tags.rs:150` (`validate_tag_name`, 558e22af) — `-x` and `HEAD` pass validation and fail only inside git, after annotate mode has been entered — suggested fix: use `git2::Tag::is_valid_name` with the existing whitespace rule.
Kind: bug
Review label: coder and validator miss. 8G 1b: "is an invalid ref: blocked".
Found during: Phase 8 review (Claude, code trace, not executed).

## Reproduction Steps
From a code trace, not executed. Use a throwaway repo.
1. Open the tag popup, type `-x`. The submit hint is enabled.

## Fix Log & Agent Notes
