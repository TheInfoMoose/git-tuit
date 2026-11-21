## Problem Description
`.github/workflows/ci.yml:25-277`, `.github/dependabot.yml:6`, `Cargo.toml:7,17-24` — Cache key drift, `rustup override` vs `rust-toolchain.toml`, member `rust-version`, `invalidstring` workspace member — suggested fix: Standardize `shared-key`, verify checkout action, optionally set `rust-version.workspace = true`
Found during: STEP 1A audit (`AUDIT_REPORT.md` finding `P2-01`). Planned phase: 2.

## Reproduction Steps
n/a (planned work)

## Fix Log & Agent Notes
