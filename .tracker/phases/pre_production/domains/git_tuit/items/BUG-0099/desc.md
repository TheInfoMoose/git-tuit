## Problem Description
CHANGELOG.md:18 says a base-less theme.ron "now fills unset fields from the classic colors instead of amber". Released gitui never had amber and base-less already filled from classic at launch, so the line describes an unreleased intermediate state. The real user-visible change, amber as the default on truecolor terminals with no theme.ron, has no CHANGELOG entry. May overlap TASK-0010 (document default theme).
Found during: Phase 14 review.

## Reproduction Steps
grep amber CHANGELOG.md: only the new line; no entry for the amber default.

## Fix Log & Agent Notes
