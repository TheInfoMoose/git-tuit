## Problem Description
STEP 2F spec gap: `[POST_GATE]` uses `grep -qE '^\tpub time: Zoned,$' src/components/utils/logitems.rs`, which fails under GNU `grep -E` because POSIX ERE treats `\t` as literal `t` rather than a tab character (`0x09`), whereas `rustfmt.toml` enforces `hard_tabs = true` so line 15 begins with a literal tab (`^Ipub time: Zoned,$`). Use `grep -qP '^\tpub time: Zoned,$'` or `grep -qE '^[[:blank:]]+pub time: Zoned,$'` instead. Location: `src/components/utils/logitems.rs:15`.
Lapse: L1 marker grep
Kind: scope-lapse
Found during: STEP 2F (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
