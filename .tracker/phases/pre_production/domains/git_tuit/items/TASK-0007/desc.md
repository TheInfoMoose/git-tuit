## Problem Description
`src/ui/mod.rs`, `src/ui/scrolllist.rs:68-74`, `src/components/mod.rs:287-301`, `src/popups/mod.rs:88-93` — No shared `Block` helper; mixed `Plain`/`Thick` borders & unpadded titles — suggested fix: Add `ui::styled_block`, `ui::popup_block`, `ui::attention_block` (`BorderType::Plain`, padded `" {title} "`, focused/unfocused tokens)
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P3-03`). Planned phase: 7.

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
