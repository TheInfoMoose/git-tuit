## Problem Description
STEP 4E spec gap: MUTATION_SPEC item 1 and POST_GATE require `match signing_key.strip_prefix("key::")` on a single line (`grep -qF 'match signing_key.strip_prefix("key::")' asyncgit/src/sync/sign.rs`), vs `asyncgit/src/lib.rs:21` denies `clippy::nursery` (`clippy::option_if_let_else` rejects `match signing_key.strip_prefix("key::") { Some(..) => .., None => .. }`) and `rustfmt.toml` (`max_width = 70`) wraps `match signing_key.strip_prefix("key::")` onto two lines. Using `strip_prefix("key::").map(str::to_string).map_or_else(...)` satisfies both clippy and rustfmt, but fails the literal `match signing_key.strip_prefix("key::")` POST_GATE grep. Location: `asyncgit/src/sync/sign.rs:208`.
Lapse: L1 marker grep
Kind: scope-lapse
Found during: STEP 4E (Neon)

## Reproduction Steps
n/a (spec gap)

## Fix Log & Agent Notes
