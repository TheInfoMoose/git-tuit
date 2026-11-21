## Problem Description
`.github/workflows/ci.yml:101-157` — `build-linux-musl` matrix, `apt-get`, `rustup target` order, `cargo install` — suggested fix: Scope to `stable`, run `apt-get update && apt-get install -y musl-tools gpgsm`, fix target setup & drop non-target install
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P0-04`). Planned phase: 2.

## Reproduction Steps
1. Inspect `.github/workflows/ci.yml:101-157` (static audit finding; see `AUDIT_REPORT.md` P0-04).
2. Observe: `build-linux-musl` matrix, `apt-get`, `rustup target` order, `cargo install`

## Fix Log & Agent Notes
