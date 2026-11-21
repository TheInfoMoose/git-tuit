## Problem Description
`src/ui/style.rs:12-39,286-290,330-361`, `THEMES.md` — Legacy ANSI `Theme::default()` & pre-0.23 migration hazard — suggested fix: Apply Nebulous/Orrery 7-color P3 Amber-on-Black palette (`#000000`, `#0C0800`, `#FFB000`, `#FFD866`, `#996300`, `#5C3A00`, `#FF5500`); preserve 24 fields + `"¶"`; add `#[serde(default)]`; update `THEMES.md`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P3-01`). Planned phase: 7.

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
