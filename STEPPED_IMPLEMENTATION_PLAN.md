# `git-tuit` Stepped Implementation Plan

**Goal:** fix what the STEP 1A audit found in this gitui fork (`AUDIT_REPORT.md`, 73 findings), in phases: Linux-only CI and supply chain, a green lint/test gate, security hardening, crash and data-loss fixes, usability, then the Nebulous/Orrery amber CRT theme. Keep the `gitui` binary name and `~/.config/gitui/` compatibility throughout.

**Audit reference:** [`AUDIT_REPORT.md`](AUDIT_REPORT.md). Every finding (`P0-01` … `P3-08`) has a pending neb item; `.phased/finding-items.json` maps finding ID → neb ID.

## Locked Decisions

- **CI scope:** Linux-only (`ubuntu-latest`): `stable` + MSRV `1.88`, `x86_64-unknown-linux-musl`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `tombi format --check`, `cargo deny check`. Upstream-only workflows (`nightly.yml`, `brew.yml`) and non-Linux jobs are removed.
- **`chrono` advisory (`RUSTSEC-2026-0543`):** migrate the `chrono` usage sites to `jiff` (`0.2.32`, already in `Cargo.lock`). No `deny.toml` ignore.
- **No new advisory ignores.** If an advisory can't be fixed by an update inside a step's scope, the step reports `Blocked`. One reviewed exception (Phase 2 review, BUG-0060): heapless 0.8.0 RUSTSEC-2026-0574..0582, compiled via faster-hex but unreachable because gix-hash enables faster-hex `std`.
- **Compatibility:** keep crate/binary name `gitui`, `~/.config/gitui/` configuration, keybindings and `theme.ron` override compatibility.
- **Styling tokens (from the Nebulous and Orrery TUIs), for Phase 7:**
  - 7-color P3 phosphor amber-on-black base: `BG_BLACK` (`#000000`), `PANEL_BG` (`#0C0800`), `AMBER_PRIMARY` (`#FFB000`), `AMBER_BRIGHT` (`#FFD866`), `AMBER_DIM` (`#996300`), `AMBER_DARK` (`#5C3A00`), `AMBER_ALERT` (`#FF5500`).
  - **BorderType:** `BorderType::Plain` (`Borders::ALL`, `┌─┐│└─┘`) with padded `" {title} "` titles.
  - **Diff/status accents:** warm phosphor-green additions (`#7FD962`) + `AMBER_ALERT` (`#FF5500`) deletions.
  - **Canvas background:** full-frame `BG_BLACK` (`#000000`).
  - **Selection background:** lifted dark amber (`#2A1A00` / `Color::Rgb(42, 26, 0)`).
- **Formatting rules:** `rustfmt.toml` (`max_width = 70`, `hard_tabs = true`, `newline_style = "Unix"`), `.clippy.toml` (`msrv = "1.88.0"`), `tombi.toml` (`line-width = 220`).

---

## Step Execution Roadmap

**Workflow:** phased

**Base gate:** `cargo fmt --all -- --check && cargo build --workspace --all-targets --locked`

**Forbidden paths:** `AUDIT_REPORT.md:CLAUDE.md:.orrery/*:scripts/*:wix/*:docs/*`

**Planning files:** `AUDIT_REPORT.md`

**Base gate note:** Phase 2's Base gate compiles everything but doesn't run clippy or tests, because HEAD doesn't pass `clippy --all-targets -D warnings` yet (audit `P0-15`: >1,270 test-module diagnostics). This is a temporary weaker gate: it hides existing clippy and test failures until Phase 3. Phase 3 steps add per-crate clippy checks on top, and STEP 3K runs the full gate. The Phase 3 review switches the Base gate to `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.

**Gate environment:** POST_GATE runs twice: in the agent container (Neon/Hydrogen) and on the host (`phased:step_check`, authoritative). The container has no PyYAML, so YAML parse checks are written as `{ ! python3 -c 'import yaml' 2>/dev/null || <parse>; }`: skipped in the container, enforced on the host. The toolchain is pinned in `rust-toolchain.toml` (STEP 2J) so `cargo fmt` gives the same result in both places.

**How to read this:** work goes strictly top to bottom through **Up Next**, one step at a time. The **#** column is the only order. Step IDs are kept because commits and notes cite them. A step whose "Waits on" names an Open Question or a **[USER]** action doesn't start until that's resolved.

**Who runs what (phased workflow, through Orrery `gemini-phased`):**
- **Claude** writes the phase (full step blocks) and reviews it once at the end. It never builds or tests code here.
- **Gemini Hydrogen** (`agy-hydrogen-orch`, stage 0) routes the step; **Gemini Neon** (`agy-neon-impl`, stage 1) executes it; **Hydrogen** (`agy-hydrogen-val`, stage 2) reads the diff. Orrery's `phased:step_start` hook runs PRE_GATE, `neb start` and the snapshot; the `phased:step_check` gate runs `scripts/phased-step.sh check`; `phased:step_commit` commits on pass; `phased:step_defer` saves the attempt and restores the tree on deferral. Nothing failing is committed.
- **Neon** never commits, never edits this file, and never edits the forbidden paths. It files bugs and scope lapses as **pending** neb items.
- **Two failures** on one step → the step is deferred. A step whose PRE_GATE then fails is deferred without running. Too many deferrals (`max(2, ⌈steps/3⌉)`) stop the phase early.

**Phase 3 goal:** `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass reliably on the pinned toolchain, `cargo deny check` fails on transitive unsound crates and passes cleanly, and the Linux-only CI workflow (deferred 2A) lands.

_Phase 2 goal (done, reviewed 2026-11-21):_ the GitHub workflows are Linux-only and correct, `deny.toml` works with cargo-deny 0.20, `chrono` is replaced by `jiff`, `cargo deny check` passes, and the `Makefile` matches CI. Everything but 2A landed; 2A is re-queued as 2K.

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = step ID); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 2K** | `/goal` | Linux-only CI workflow (re-queued 2A) | `.github/workflows/ci.yml` | — |
| 2 | **STEP 3A** | `/goal` | deny.toml unsound scope, heapless ignores, dead skip-tree | `deny.toml` | — |
| 3 | **STEP 3B** | `/goal` | Allow unwrap/expect/panic in tests | `.clippy.toml` | — |
| 4 | **STEP 3C** | `/goal` | git2-hooks clippy and python3 hook tests | `git2-hooks/src/lib.rs` | — |
| 5 | **STEP 3J** | `/goal` | gitui clippy: app, options, diff | `src/app.rs`, `src/options.rs`, `src/components/diff.rs` | — |
| 6 | **STEP 3D** | `/goal` | asyncgit clippy: blame, commit, commit_details | `asyncgit/src/sync/{blame,commit,commit_details}.rs` | — |
| 7 | **STEP 3E** | `/goal` | asyncgit clippy: logwalker, sync/mod, merge_commit | `asyncgit/src/sync/logwalker.rs`, `asyncgit/src/sync/mod.rs`, `asyncgit/src/sync/branch/merge_commit.rs` | — |
| 8 | **STEP 3F** | `/goal` | asyncgit clippy: merge_ff, commits_info, hooks | `asyncgit/src/sync/branch/merge_ff.rs`, `asyncgit/src/sync/commits_info.rs`, `asyncgit/src/sync/hooks.rs` | — |
| 9 | **STEP 3G** | `/goal` | asyncgit clippy: cred, sign, stash | `asyncgit/src/sync/{cred,sign,stash}.rs` | — |
| 10 | **STEP 3H** | `/goal` | asyncgit clippy: remotes push and tags | `asyncgit/src/sync/remotes/{push,tags}.rs` | — |
| 11 | **STEP 3I** | `/boost` | GIT_DIR test in a child process, GNUPGHOME guard | `asyncgit/src/revlog.rs`, `asyncgit/src/sync/sign.rs` | — |
| 12 | **STEP 3K** | `/goal` | gitui clippy: reflow, key_config; full gate | `src/ui/reflow.rs`, `src/keys/key_config.rs` | — |

### Completed

| Step ID | Status | Command | Scope | Target Files |
| :--- | :---: | :---: | :--- | :--- |
| STEP 1A | ✅ | — | Parallel subagent audit (Gemini) | `AUDIT_REPORT.md` |
| STEP 2J | ✅ 7d5fd362 | `/goal` | Pin the Rust toolchain to 1.99.1 | `rust-toolchain.toml` |
| STEP 2A | ↪ withdrawn | `/goal` | Deferred at start (dirty `ci.yml`); re-queued as STEP 2K | `.github/workflows/ci.yml` |
| STEP 2B | ✅ 22d64e25 | `/goal` | Remove upstream workflows, Linux-only release | `.github/workflows/cd.yml`, `nightly.yml`, `brew.yml` |
| STEP 2C | ✅ e4fcbcee | `/goal` | Workspace manifests and dependabot cadence | `Cargo.toml`, member `Cargo.toml`s, `.github/dependabot.yml` |
| STEP 2D | ✅ 3956e754 | `/goal` | `deny.toml` for cargo-deny 0.20 | `deny.toml` |
| STEP 2E | ✅ 78aca30f | `/goal` | jiff dependency, `build.rs` and `time_to_string` | `Cargo.toml`, `build.rs`, `src/components/utils/mod.rs` |
| STEP 2F | ✅ 60dd82f4 | `/goal` | jiff in `LogEntry` and its callers (L1 amendment ratified) | `src/components/utils/logitems.rs`, `src/components/commitlist.rs`, `src/popups/file_revlog.rs` |
| STEP 2G | ✅ 72ded3fd | `/goal` | Drop `chrono` | `Cargo.toml`, `Cargo.lock` |
| STEP 2H | ✅ 16b2a7b8 | `/goal` | Lockfile advisory updates (+ ratatui-widgets 0.3.2 for strum dedupe) | `Cargo.lock`, `asyncgit/Cargo.toml` |
| STEP 2I | ✅ 3afc634c | `/goal` | `Makefile` matches Linux CI | `Makefile`, `.cargo/config.toml` |

### User Step: FETCH

Warm the host cache so the host-side `phased:step_check` gate can build, then capture the baseline Phase 3 is written from (Claude can't run cargo in this repo):

```bash
cargo fetch --locked
```

```bash
mkdir -p docs/baseline && cargo clippy --workspace --all-targets --keep-going -- -D warnings 2>&1 | tee docs/baseline/clippy.txt; cargo test --workspace --no-fail-fast 2>&1 | tee docs/baseline/test.txt
```

### Future phases

Outlines only (goal + step titles). Full step blocks are written at the phase review before each phase starts.

- **Phase 4 — Security hardening.** Goal: no command execution, file read or credential leak driven by an untrusted repo's config, paths or refs.
  - Global-only git config reader (P1-12)
  - Credential helper from global config, URL/newline validation (P1-02)
  - Redact credentials in `Debug` and logs (P1-04)
  - Expected-URL check and per-remote credentials (P1-03, 2 steps)
  - Hook path expansion, name and exec checks; `--` in the sh fallback (P1-05, P1-06)
  - Safe `COMMIT_EDITMSG` in git2-hooks (P1-07); `create_hook` without chmod/unwrap (P1-08)
  - pre-push ref validation (P1-10)
  - Signing program and key validation (P1-09, 2 steps)
  - `repo_write_file` traversal/symlink guard (P1-11)
  - External editor: source, containment, tokenizer, drop guard, exit status (P1-13..P1-16, P2-05; 3 steps)
  - Clipboard tool resolution and OSC 52 cap (P1-17, P2-03)
  - CLI args: theme name, log file perms, `-d`/`-w` (P1-18, P1-19, P2-04; 2 steps)
  - Credential popup clears inputs (P1-20); commit popup edit file and template (P1-21)
  - Error state getters on `AsyncFetchJob` / `AsyncBlame` (prep for P2-21)
- **Phase 5 — Crashes and data loss.** Goal: no handler error exits the TUI, no audited panic remains, and no destructive action runs without a correct target and confirmation.
  - Delete the right remote branch (P1-22)
  - Fatal `?` → error popups (P1-23, 3 steps)
  - Underflow/OOB/`expect` panics (P1-24, 4 steps)
  - Credential-prompt cancel aborts push/pull/fetch; preflight modal lockup (P2-10, 2 steps)
  - Sticky hard-reset/discard defaults (P2-06)
  - Confirm prompts name targets and warn on untracked deletes (P2-07)
  - Confirms for revert, merge, rebase, detached checkout, submodule update (P2-08, 2 steps)
- **Phase 6 — Usability and modal flow.** Goal: popups take events in draw order, nothing leaks keys or gets stuck, errors are shown. P2-09, P2-11..P2-23, one finding (or half of one) per step, ≤3 files each (~14 steps).
- **Phase 7 — Amber CRT theme and chrome.** Goal: the Locked Decisions' styling tokens are the default theme, with `theme.ron` overrides still working.
  - Default palette + `#[serde(default)]` on `Theme` (P3-01)
  - Semantic style helpers + `THEMES.md` (P3-02)
  - Shared plain-border block helpers (P3-03)
  - Frame paint, tab bar, spinner (P3-04)
  - Command bar layout and styling (P3-05, P2-24)
  - Pane and popup migrations (P3-06, P3-07; ~5 steps)
  - Snapshot updates + README note (P3-08)

### Open Questions

_None yet._

### Parked

Tracked in neb: `neb list --status pending`. Reviewed at phase end; only picked items become steps.

---

## Phase 2: Linux-only CI and supply chain

### STEP 2J: Pin the Rust Toolchain

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2J: Pin rust-toolchain.toml to 1.99.1 so rustfmt, clippy and cargo give the same result on the host gate, in the agent container and in CI's lint job.

[SCOPE_LOCK]
ALLOWED_FILES:
  - rust-toolchain.toml
READ_ONLY_REFS:
  - rust-toolchain.toml:1-3 (current: channel = "stable", profile = "default")
  - Agent image toolchain is named `1.99.1-x86_64-unknown-linux-gnu`; floating `stable` pulled 1.100.0, whose rustfmt reformats asyncgit/src/sync/reword.rs:32, src/app.rs:1182, src/components/commitlist.rs:354 and src/popups/branchlist.rs:418 (TASK-0020).
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't reformat any .rs file.

[PRE_GATE]
true

[MUTATION_SPEC]
1. rust-toolchain.toml: replace the whole file with exactly:

[toolchain]
channel = "1.99.1"
profile = "default"
components = ["clippy", "rustfmt"]

   Why: `stable` floats to whatever rustup last installed, so fmt results differ between environments. 1.99.1 is the toolchain in the agent image and on the host. CI's build/test matrix still chooses its own toolchain through RUSTUP_TOOLCHAIN.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml && grep -qF 'components = ["clippy", "rustfmt"]' rust-toolchain.toml && rustc --version | grep -qF 'rustc 1.99.1'

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 2A: Linux-Only CI Workflow

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2A: Replace .github/workflows/ci.yml with a Linux-only workflow (stable + MSRV 1.88 build/test/clippy, musl build/test, fmt/tombi/cargo-deny lints) so CI stops running upstream-only jobs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - .github/workflows/ci.yml
READ_ONLY_REFS:
  - AUDIT_REPORT.md:61-73 (P0-01..P0-06, P1-01, P2-01)
  - rust-toolchain.toml (pinned to channel = "1.99.1" by STEP 2J; the RUSTUP_TOOLCHAIN env var overrides it, which is how the stable and 1.88 matrix legs pick their toolchain; the lint job sets no RUSTUP_TOOLCHAIN so rustfmt matches the pin)
  - Makefile:55-62 (musl build targets; not used by the new workflow)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't edit Makefile, cd.yml, nightly.yml or brew.yml here (later steps).

[PRE_GATE]
grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml

[MUTATION_SPEC]
1. .github/workflows/ci.yml: replace the whole file with exactly this content (2-space YAML indentation, LF line endings, trailing newline):

name: CI

on:
  push:
    branches: [master]
  pull_request:
    branches: [master]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: Build (${{ matrix.rust }})
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        rust: [stable, "1.88"]
    env:
      RUSTUP_TOOLCHAIN: ${{ matrix.rust }}
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          components: clippy

      - name: Restore cargo cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: ci-${{ matrix.rust }}

      # gpgsm: needed by the x509 signing e2e test
      - name: Install commit-signing tools
        run: sudo apt-get update && sudo apt-get install -y gpgsm

      - name: Build Debug
        run: cargo build --workspace --locked

      - name: Run tests
        run: cargo test --workspace --locked

      - name: Run clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Build Release
        run: cargo build --release --locked

      - name: Binary Size
        run: ls -l ./target/release/gitui

  build-linux-musl:
    name: Build musl
    runs-on: ubuntu-latest
    env:
      RUSTUP_TOOLCHAIN: stable
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-musl

      - name: Restore cargo cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: ci-musl

      # gpgsm: needed by the x509 signing e2e test
      - name: Install musl and commit-signing tools
        run: sudo apt-get update && sudo apt-get install -y musl-tools gpgsm

      - name: Build Debug
        run: |
          cargo build --target x86_64-unknown-linux-musl --locked
          ./target/x86_64-unknown-linux-musl/debug/gitui --version

      - name: Build Release
        run: |
          cargo build --release --target x86_64-unknown-linux-musl --locked
          ./target/x86_64-unknown-linux-musl/release/gitui --version
          ls -l ./target/x86_64-unknown-linux-musl/release/gitui

      - name: Run tests
        run: cargo test --workspace --target x86_64-unknown-linux-musl --locked

  linting:
    name: Lints
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # no RUSTUP_TOOLCHAIN here: rustfmt must match rust-toolchain.toml
      - name: Install Rust
        uses: dtolnay/rust-toolchain@1.99.1
        with:
          components: rustfmt

      - name: Restore cargo cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: ci-lint

      - name: rustfmt
        run: cargo fmt --all -- --check

      - name: tombi install
        uses: tombi-toml/setup-tombi@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          version: '0.9.0'

      - name: tombi check
        run: tombi format --check

      - name: cargo-deny install
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-deny@0.20.2

      - name: cargo-deny checks
        run: cargo deny check

   Why: removes the daily cron (P0-01), non-Linux/nightly legs (P0-02), `make clippy` (P0-03), the broken musl matrix/apt/target/install steps (P0-04), the five upstream-only jobs (P0-05), fmt without --all (P0-06), the unpinned source-built cargo-deny (P1-01), and the cache-key / `rustup override` drift (P2-01). The clippy and deny steps will stay red in CI until Phases 2–3 land; that is expected.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && { ! python3 -c 'import yaml' 2>/dev/null || python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml"))'; } && ! grep -qE '^[[:space:]]+schedule:' .github/workflows/ci.yml && ! grep -qE 'macos-latest|windows-latest|nightly|armkeil|actions-rs/|rustup override|make clippy' .github/workflows/ci.yml && grep -qE 'rust: \[stable, "1\.88"\]' .github/workflows/ci.yml && grep -qF 'cargo clippy --workspace --all-targets -- -D warnings' .github/workflows/ci.yml && grep -qF 'cargo fmt --all -- --check' .github/workflows/ci.yml && grep -qF 'tool: cargo-deny@0.20.2' .github/workflows/ci.yml && grep -qF 'uses: dtolnay/rust-toolchain@1.99.1' .github/workflows/ci.yml && grep -qF 'sudo apt-get update && sudo apt-get install -y musl-tools gpgsm' .github/workflows/ci.yml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 2B: Remove Upstream Workflows and Make Release Linux-Only

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2B: Delete the upstream nightly and Homebrew workflows and replace cd.yml with a Linux-only (musl) tag release, so no workflow needs upstream secrets or non-Linux runners.

[SCOPE_LOCK]
ALLOWED_FILES:
  - .github/workflows/cd.yml
  - .github/workflows/nightly.yml (DELETE)
  - .github/workflows/brew.yml (DELETE)
READ_ONLY_REFS:
  - AUDIT_REPORT.md:69-71 (P0-07..P0-09)
  - Makefile:44-59 (release-linux-musl → build-linux-musl-release; both targets stay in STEP 2I)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't edit ci.yml or Makefile.

[PRE_GATE]
grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml

[MUTATION_SPEC]
1. Delete `.github/workflows/nightly.yml` (P0-07: upstream S3 nightly cron with missing secrets).
2. Delete `.github/workflows/brew.yml` (P0-08: upstream Homebrew bump with missing BREW_TOKEN).
3. .github/workflows/cd.yml: replace the whole file with exactly this content (P0-09; drops the workflow_dispatch trigger, which had no tag for gh-release):

name: CD

on:
  push:
    tags:
      - "*"

permissions:
  contents: write

jobs:
  release:
    runs-on: ubuntu-latest
    env:
      RUSTUP_TOOLCHAIN: stable
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-musl

      - name: Restore cargo cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: cd-musl

      # gpgsm: needed by the x509 signing e2e test
      - name: Install musl and commit-signing tools
        run: sudo apt-get update && sudo apt-get install -y musl-tools gpgsm

      - name: Run tests
        run: cargo test --workspace --locked

      - name: Build Release Linux
        env:
          GITUI_RELEASE: 1
        run: make release-linux-musl

      - name: Extract release notes
        id: release_notes
        uses: ffurrer2/extract-release-notes@v2

      - name: Release
        uses: softprops/action-gh-release@v2
        with:
          body: ${{ steps.release_notes.outputs.release_notes }}
          prerelease: ${{ contains(github.ref, '-') }}
          files: |
            ./release/*.tar.gz

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && test ! -e .github/workflows/nightly.yml && test ! -e .github/workflows/brew.yml && { ! python3 -c 'import yaml' 2>/dev/null || python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/cd.yml"))'; } && ! grep -qE 'macos|windows|ubuntu-22\.04|homebrew|BREW_TOKEN|armkeil|workflow_dispatch|apt-get -qq' .github/workflows/cd.yml && grep -qF 'run: make release-linux-musl' .github/workflows/cd.yml && grep -qF 'sudo apt-get update && sudo apt-get install -y musl-tools gpgsm' .github/workflows/cd.yml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 2C: Workspace Manifests and Dependabot Cadence

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2C: Add invalidstring to the workspace members, give every member crate rust-version = "1.88", and move dependabot to a weekly cadence (P2-01).

[SCOPE_LOCK]
ALLOWED_FILES:
  - Cargo.toml
  - Cargo.lock
  - asyncgit/Cargo.toml
  - filetreelist/Cargo.toml
  - git2-hooks/Cargo.toml
  - git2-testing/Cargo.toml
  - scopetime/Cargo.toml
  - invalidstring/Cargo.toml
  - .github/dependabot.yml
READ_ONLY_REFS:
  - Cargo.toml:1-24 ([package] rust-version = "1.88" at :7; [workspace] members at :17-24)
  - asyncgit/Cargo.toml:39 (invalidstring is already a dev-dependency path crate)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No dependency changes. These are mechanical manifest edits, which is why this step lists more than 3 files.

[PRE_GATE]
grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml

[MUTATION_SPEC]
1. Cargo.toml [workspace] members: anchor `  "git2-testing",` → insert `  "invalidstring",` on the next line, so the list reads asyncgit, filetreelist, git2-hooks, git2-testing, invalidstring, scopetime (alphabetical).
2. Each of asyncgit/Cargo.toml, filetreelist/Cargo.toml, git2-hooks/Cargo.toml, git2-testing/Cargo.toml, scopetime/Cargo.toml, invalidstring/Cargo.toml: anchor `edition = "2021"` in [package] → insert `rust-version = "1.88"` on the line directly after it.
3. .github/dependabot.yml: `    interval: daily` → `    interval: weekly`.
4. Cargo.lock: if `cargo build` rewrites it after adding the member, keep that change; no other lockfile edits.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && grep -qE '^  "invalidstring",$' Cargo.toml && for f in asyncgit filetreelist git2-hooks git2-testing scopetime invalidstring; do grep -qE '^rust-version = "1\.88"$' $f/Cargo.toml || exit 1; done && grep -qE '^    interval: weekly$' .github/dependabot.yml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 2D: `deny.toml` for cargo-deny 0.20

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2D: Migrate deny.toml to cargo-deny 0.20 syntax, prune stale skip-tree entries and add [graph]/[sources], so `cargo deny check bans licenses sources` passes.

[SCOPE_LOCK]
ALLOWED_FILES:
  - deny.toml
READ_ONLY_REFS:
  - deny.toml:1-63 (current file)
  - AUDIT_REPORT.md:94,97 (P0-14, P2-02)
  - Cargo.lock (single-version crates on HEAD: unicode-truncate 2.0.1, redox_syscall 0.5.8, sha1-checked 0.10.0 → their skip-tree entries are stale)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't add advisory ignores. Keep the existing RUSTSEC-2025-0141 ignore and the license allow list unchanged.

[PRE_GATE]
grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml

[MUTATION_SPEC]
1. [advisories]: delete the line `version = 2`. Keep the `ignore = [...]` entry for RUSTSEC-2025-0141 as is.
2. [bans] skip-tree: change every `{ name = "<crate>" }` to `{ crate = "<crate>" }`, keeping each entry's comment.
3. [bans] skip-tree: delete the entries (and their comment lines) for `unicode-truncate`, `redox_syscall` and `sha1-checked`. Remaining entries: windows-sys, unicode-width, thiserror, windows-core, hashbrown, getrandom, rustix.
4. Add at the top of the file, before [licenses]:

[graph]
targets = ["x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"]

5. Add at the end of the file:

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]

6. Run `cargo deny check bans licenses sources`. If bans reports a duplicate that a removed skip-tree entry covered, or a new duplicate, add a `{ crate = "<name>" }` skip-tree entry for it with a `# currently needed due to:` comment naming the two dependents from `cargo tree -i <name>`. Report every entry added this way in your summary.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo deny check bans licenses sources && ! grep -qE '^version = 2$' deny.toml && ! grep -qE '\{ name = "' deny.toml && ! grep -qE '"(unicode-truncate|redox_syscall|sha1-checked)"' deny.toml && grep -qE '^unknown-git = "deny"$' deny.toml && grep -qE '^targets = \["x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"\]$' deny.toml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 2E: jiff Dependency, `build.rs` and `time_to_string`

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2E: Add jiff 0.2 as a dependency and build-dependency, port build.rs and utils::time_to_string from chrono to jiff with identical output, and remove build.rs's panicking slice/unwrap paths.

[SCOPE_LOCK]
ALLOWED_FILES:
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src/components/utils/mod.rs
READ_ONLY_REFS:
  - build.rs:1-49 (whole file)
  - src/components/utils/mod.rs:1,29-45 (use line; fn time_to_string)
  - Cargo.toml:35,45-46,75-76 (chrono dep; itertools/log neighbours; [build-dependencies])
  - Cargo.lock (jiff 0.2.32 is already locked via gix-date and env_logger)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep the `chrono` dependency lines in Cargo.toml (STEP 2G removes them; logitems.rs still uses chrono until STEP 2F). Keep the signature `pub fn time_to_string(secs: i64, short: bool) -> String`.

[PRE_GATE]
grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml

[MUTATION_SPEC]
1. Cargo.toml [dependencies]: insert `jiff = "0.2"` on the line after `itertools = "0.14"` (alphabetical order for tombi).
2. Cargo.toml [build-dependencies]: insert `jiff = "0.2"` above the existing `chrono = ...` line.
3. Cargo.lock: run `cargo build --workspace` once without `--locked` to record the new edges. jiff must stay at 0.2.32: `grep -A1 '^name = "jiff"$' Cargo.lock` shows `version = "0.2.32"`. No other version changes.
4. build.rs: delete line 1 `use chrono::TimeZone;`.
5. build.rs get_git_hash: replace
	if let Ok(commit) = std::env::var("BUILD_GIT_COMMIT_ID") {
		return commit[..7].to_string();
	};
   with
	if let Ok(commit) = std::env::var("BUILD_GIT_COMMIT_ID") {
		return commit.chars().take(7).collect();
	};
   Why: `commit[..7]` panics on ids shorter than 7 bytes or with a multi-byte char at the boundary (P2-02).
6. build.rs get_git_hash: replace the final `panic!("Can not get git commit: {}", commit.unwrap_err());` with
	println!(
		"cargo:warning=can not get git commit: {}",
		commit.unwrap_err()
	);
	"unknown".into()
   Why: a source tarball without git must still build (P2-02).
7. build.rs main: replace the whole `let now = match ... ;` + `let build_date = now.date_naive();` (lines 28-34) with
	let build_date = std::env::var("SOURCE_DATE_EPOCH")
		.ok()
		.and_then(|val| val.parse::<i64>().ok())
		.and_then(|secs| jiff::Timestamp::from_second(secs).ok())
		.map_or_else(
			|| jiff::Zoned::now().date(),
			|ts| ts.to_zoned(jiff::tz::TimeZone::system()).date(),
		);
   `build_date` is a `jiff::civil::Date`; its Display prints `YYYY-MM-DD`, same as chrono's NaiveDate. An unparsable SOURCE_DATE_EPOCH now falls back to the current date instead of panicking (P2-02).
8. build.rs main: wrap the warning line so it only prints for nightly builds: replace `println!("cargo:warning=buildname '{build_name}'");` with
	if std::env::var("GITUI_RELEASE").is_err() {
		println!("cargo:warning=buildname '{build_name}'");
	}
9. src/components/utils/mod.rs: delete `use chrono::{DateTime, Local, Utc};`.
10. src/components/utils/mod.rs time_to_string: replace the body with
	let time = jiff::Timestamp::from_second(secs)
		.unwrap_or(jiff::Timestamp::UNIX_EPOCH)
		.to_zoned(jiff::tz::TimeZone::system());

	time.strftime(if short {
		"%Y-%m-%d"
	} else {
		"%Y-%m-%d %H:%M:%S"
	})
	.to_string()
   Why: chrono's `from_timestamp(..).unwrap_or_default()` fell back to the epoch too; output format is unchanged.
11. Tests: add at the end of src/components/utils/mod.rs (create `#[cfg(test)] mod tests { use super::*; ... }` if the file has none; otherwise append inside the existing one):
	#[test]
	fn time_to_string_formats_short_and_long() {
		let short = time_to_string(0, true);
		let long = time_to_string(0, false);
		assert_eq!(short.len(), 10);
		assert_eq!(long.len(), 19);
		assert!(long.starts_with(&short));
	}
	#[test]
	fn time_to_string_out_of_range_falls_back() {
		assert_eq!(time_to_string(i64::MAX, false), time_to_string(0, false));
	}

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo test -p gitui --locked time_to_string && grep -qE '^jiff = "0\.2"$' Cargo.toml && test "$(grep -cE '^jiff = "0\.2"$' Cargo.toml)" = 2 && grep -A1 '^name = "jiff"$' Cargo.lock | grep -qF 'version = "0.2.32"' && ! grep -qE '\bchrono\b' build.rs src/components/utils/mod.rs && ! grep -qF 'commit[..7]' build.rs && ! grep -qE '\bpanic!' build.rs && grep -qF 'jiff::Timestamp::from_second(secs)' src/components/utils/mod.rs && grep -qE '\bfn time_to_string_out_of_range_falls_back\b' src/components/utils/mod.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 2F: jiff in `LogEntry` and Its Callers

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2F: Store LogEntry.time as jiff::Zoned and pass `now: &jiff::Zoned` through CommitList and FileRevlogPopup, keeping the same "<1m ago" / "NNm ago" / "HH:MM:SS" / "YYYY-MM-DD" output.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/utils/logitems.rs
  - src/components/commitlist.rs
  - src/popups/file_revlog.rs
READ_ONLY_REFS:
  - src/components/utils/logitems.rs:1-78 (use line; LogEntry; From<CommitInfo>; LogEntry::time_to_string)
  - src/components/commitlist.rs:20,443-454,520-525,568-620 (use line; get_entry_to_add `now` param; e.time_to_string(now); get_text `let now = Local::now();` and the get_entry_to_add call)
  - src/popups/file_revlog.rs:21,266,277,384-387 (use line; get_rows; entry.time_to_string(now); draw)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't change Cargo.toml (STEP 2E added jiff; STEP 2G drops chrono). Keep LogEntry's other fields and the output strings unchanged.

[PRE_GATE]
grep -qF 'jiff::Timestamp::from_second(secs)' src/components/utils/mod.rs

[MUTATION_SPEC]
1. logitems.rs: replace `use chrono::{DateTime, Duration, Local, Utc};` with `use jiff::{tz::TimeZone, Timestamp, Zoned};`.
2. logitems.rs LogEntry: `pub time: DateTime<Local>,` → `pub time: Zoned,`.
3. logitems.rs From<CommitInfo>: replace the whole `let time = { ... };` block with
		let time = Timestamp::from_second(c.time)
			.unwrap_or_else(|_| {
				log::error!("error reading commit date: {hash_short} - timestamp: {}",c.time);
				Timestamp::UNIX_EPOCH
			})
			.to_zoned(TimeZone::system());
4. logitems.rs LogEntry::time_to_string: signature → `pub fn time_to_string(&self, now: &Zoned) -> String`, body →
		let delta = now.timestamp().as_second()
			- self.time.timestamp().as_second();
		if delta < 30 * 60 {
			let delta_str = if delta < 60 {
				"<1m ago".to_string()
			} else {
				format!("{:0>2}m ago", delta / 60)
			};
			format!("{delta_str: <10}")
		} else if self.time.date() == now.date() {
			self.time.strftime("%T  ").to_string()
		} else {
			self.time.strftime("%Y-%m-%d").to_string()
		}
   Why: same thresholds as the chrono version; negative deltas (commit dates in the future) still print "<1m ago".
5. Tests: append to logitems.rs (inside `#[cfg(test)] mod tests` with `use super::*;`, creating it if absent). Build entries with a helper:
	fn entry_at(secs: i64) -> LogEntry {
		LogEntry::from(CommitInfo {
			message: "msg".into(),
			time: secs,
			author: "testuser".into(),
			id: CommitId::default(),
		})
	}
	#[test]
	fn time_to_string_recent() {
		let e = entry_at(1_000_000);
		let now = Timestamp::from_second(1_000_030).unwrap().to_zoned(TimeZone::UTC);
		assert_eq!(e.time_to_string(&now), "<1m ago   ");
		let now = Timestamp::from_second(1_000_000 + 5 * 60).unwrap().to_zoned(TimeZone::UTC);
		assert_eq!(e.time_to_string(&now), "05m ago   ");
	}
	#[test]
	fn time_to_string_old_is_date() {
		let e = entry_at(0);
		let now = Timestamp::from_second(400 * 86_400).unwrap().to_zoned(TimeZone::system());
		assert_eq!(e.time_to_string(&now), e.time.strftime("%Y-%m-%d").to_string());
	}
   `CommitInfo` (asyncgit/src/sync/commits_info.rs:116-125) has exactly message, time, author, id; `CommitId::default()` is what commitlist.rs:958 uses in its tests. Add `use asyncgit::sync::{CommitId, CommitInfo};` only if the file's top-level import isn't visible through `use super::*;`.
6. commitlist.rs: replace `use chrono::{DateTime, Local};` with `use jiff::Zoned;`.
7. commitlist.rs get_entry_to_add: param `now: DateTime<Local>,` → `now: &Zoned,`. The call `e.time_to_string(now)` stays as written (now is already a reference).
8. commitlist.rs get_text: `let now = Local::now();` → `let now = Zoned::now();`, and in the get_entry_to_add call change the argument `now,` → `&now,`.
9. file_revlog.rs: replace `use chrono::{DateTime, Local};` with `use jiff::Zoned;`.
10. file_revlog.rs: `fn get_rows(&self, now: DateTime<Local>) -> Vec<Row<'_>>` → `fn get_rows(&self, now: &Zoned) -> Vec<Row<'_>>`; `entry.time_to_string(now)` stays.
11. file_revlog.rs draw: `let now = Local::now();` → `let now = Zoned::now();`; `let rows = self.get_rows(now);` → `let rows = self.get_rows(&now);`.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo test -p gitui --locked time_to_string && ! grep -qE '\bchrono\b' src/components/utils/logitems.rs src/components/commitlist.rs src/popups/file_revlog.rs && grep -qP '^\tpub time: Zoned,$' src/components/utils/logitems.rs && grep -qF 'pub fn time_to_string(&self, now: &Zoned) -> String' src/components/utils/logitems.rs && grep -qF 'let now = Zoned::now();' src/components/commitlist.rs && grep -qF 'let rows = self.get_rows(&now);' src/popups/file_revlog.rs && grep -qE '\bfn time_to_string_recent\b' src/components/utils/logitems.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 2G: Drop `chrono`

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2G: Remove chrono from Cargo.toml and Cargo.lock now that no code uses it, clearing RUSTSEC-2026-0543.

[SCOPE_LOCK]
ALLOWED_FILES:
  - Cargo.toml
  - Cargo.lock
READ_ONLY_REFS:
  - Cargo.toml:35,75-77 (chrono dependency and build-dependency lines)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No other dependency changes.

[PRE_GATE]
! grep -rqE '\bchrono::|use chrono\b' --include=*.rs src build.rs asyncgit filetreelist git2-hooks git2-testing scopetime invalidstring

[MUTATION_SPEC]
1. Cargo.toml [dependencies]: delete `chrono = { version = "0.4", default-features = false, features = ["clock"] }`.
2. Cargo.toml [build-dependencies]: delete the same `chrono = ...` line; `jiff = "0.2"` stays.
3. Cargo.lock: run `cargo build --workspace` once without `--locked` so chrono (and anything only it pulled in, such as iana-time-zone) leaves the lockfile. No other version changes.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo deny check bans licenses sources && ! grep -qE '^chrono = ' Cargo.toml && ! grep -qE '^name = "chrono"$' Cargo.lock

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 2H: Lockfile Advisory Updates

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2H: Update libz-sys, ratatui-core (lru), faster-hex (heapless), rustix and rand past their advisories, and turn off serial_test's default features (drops futures-util/pin-utils, updates scc), so `cargo deny check` passes in full.

[SCOPE_LOCK]
ALLOWED_FILES:
  - Cargo.lock
  - asyncgit/Cargo.toml
READ_ONLY_REFS:
  - AUDIT_REPORT.md:79-90 (P0-10, P0-12, P0-13 advisory table)
  - asyncgit/Cargo.toml:37-41 ([dev-dependencies] serial_test = "3.3")
  - asyncgit/src/revlog.rs:332, asyncgit/src/sync/sign.rs:414, asyncgit/src/sync/cred.rs:206 (only sync `#[serial]` is used; no async tests)
  - Patched ranges (local RustSec DB): libz-sys >= 1.1.33; lru >= 0.18.2; faster-hex >= 0.10.1; heapless >= 0.9.4; rustix 1.x >= 1.1.6; rand 0.8.x >= 0.8.6; scc >= 3.8.4; pin-utils has no fix (must leave the tree).
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No deny.toml ignores. No changes to root Cargo.toml version requirements. MSRV stays 1.88: an update whose crate needs a newer rust-version is not allowed.

[PRE_GATE]
! grep -qE '^name = "chrono"$' Cargo.lock

[MUTATION_SPEC]
1. asyncgit/Cargo.toml [dev-dependencies]: `serial_test = "3.3"` → `serial_test = { version = "3.3", default-features = false }`.
2. Cargo.lock: run exactly
	cargo update -p libz-sys -p ratatui-core -p faster-hex -p rustix@1.1.4 -p rand@0.8.5 -p serial_test
3. Check the result with `cargo deny check advisories`. If it still reports lru, heapless or scc, run `cargo tree -i <crate>` and report `Blocked` with that output (fixing them would need a semver-major bump of a direct dependency, which is outside this step).
4. Run `cargo +1.88 check --workspace --locked` if the 1.88 toolchain is installed in your environment; if it isn't installed, say so in your report. An MSRV failure means `Blocked` with the crate name.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo deny check && grep -qF 'serial_test = { version = "3.3", default-features = false }' asyncgit/Cargo.toml && ! grep -qE '^name = "pin-utils"$' Cargo.lock && ! grep -A1 -E '^name = "libz-sys"$' Cargo.lock | grep -qE 'version = "1\.1\.(2[0-9]|3[0-2])"'

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 2I: `Makefile` Matches Linux CI

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 2I: Rewrite the Makefile to the Linux-only targets CI and release use (fmt --all, clippy --all-targets -D warnings, cargo test --workspace without nextest) and delete the ARM-only .cargo/config.toml.

[SCOPE_LOCK]
ALLOWED_FILES:
  - Makefile
  - .cargo/config.toml (DELETE)
READ_ONLY_REFS:
  - Makefile:1-128 (current file)
  - .cargo/config.toml:1-8 (only ARM cross linkers)
  - .github/workflows/cd.yml (calls `make release-linux-musl`, which must keep working)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml

[MUTATION_SPEC]
1. Makefile: replace the whole file with exactly this content (recipe lines start with one TAB):

.PHONY: profile run-timing debug build-release release-linux-musl build-linux-musl-debug build-linux-musl-release test test-linux-musl fmt clippy check deny sort install install-timing licenses clean

ARGS=-l
# ARGS=-l -d ~/code/extern/kubernetes
# ARGS=-l -w ~/code/git-bare-test

profile:
	CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --features timing -- ${ARGS}

run-timing:
	cargo run --features=timing --release -- ${ARGS}

debug:
	RUST_BACKTRACE=true cargo run --features=timing -- ${ARGS}

build-release:
	cargo build --release --locked

release-linux-musl: build-linux-musl-release
	strip target/x86_64-unknown-linux-musl/release/gitui
	mkdir -p release
	tar -C ./target/x86_64-unknown-linux-musl/release/ -czvf ./release/gitui-linux-x86_64.tar.gz ./gitui

build-linux-musl-debug:
	cargo build --target=x86_64-unknown-linux-musl

build-linux-musl-release:
	cargo build --release --target=x86_64-unknown-linux-musl --locked

test:
	cargo test --workspace --locked

test-linux-musl:
	cargo test --workspace --target=x86_64-unknown-linux-musl --locked

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

check: fmt clippy test sort deny

deny:
	cargo deny check

sort:
	tombi format --check

install:
	cargo install --path "." --offline --locked

install-timing:
	cargo install --features=timing --path "." --offline --locked

licenses:
	cargo bundle-licenses --format toml --output THIRDPARTY.toml

clean:
	cargo clean

2. Delete `.cargo/config.toml` (and the `.cargo/` directory if it is then empty).

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && make -n test fmt clippy release-linux-musl >/dev/null && test ! -e .cargo/config.toml && ! grep -qE 'nextest|apple|aarch64|armv7|arm-unknown|arm-linux|wix|release-mac|release-win|--all-features' Makefile && grep -qE '^	cargo clippy --workspace --all-targets -- -D warnings$' Makefile && grep -qE '^	cargo fmt --all -- --check$' Makefile && grep -qE '^	cargo test --workspace --locked$' Makefile

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

## Phase 3: Green gate

Clippy errors and anchors below come from `docs/baseline/clippy.txt` (Rust 1.99.1, `--keep-going`). Every listed site is test-only code (inside `#[cfg(test)]`). Line numbers are from the baseline; if a line moved, find the quoted code.

**Per-crate clippy checks before a crate is fully clean** use `--message-format=short`, whose diagnostic lines start with `path:line:col:`. A step proves its own files are clean with `! cargo clippy -p <crate> --all-targets --message-format=short 2>&1 | grep -E '^<files>:'`, while other files in the crate may still have errors.

### STEP 2K: Linux-Only CI Workflow (re-queued 2A)

* **Recommended Command:** `/goal`

Re-queue of STEP 2A (deferred at start in Phase 2 because `ci.yml` was already dirty; the block is otherwise unchanged).

```text
[STEP_ID]
STEP 2K: Replace .github/workflows/ci.yml with a Linux-only workflow (stable + MSRV 1.88 build/test/clippy, musl build/test, fmt/tombi/cargo-deny lints) so CI stops running upstream-only jobs.

[SCOPE_LOCK]
ALLOWED_FILES:
  - .github/workflows/ci.yml
READ_ONLY_REFS:
  - AUDIT_REPORT.md:61-73 (P0-01..P0-06, P1-01, P2-01)
  - rust-toolchain.toml (pinned to channel = "1.99.1" by STEP 2J; the RUSTUP_TOOLCHAIN env var overrides it, which is how the stable and 1.88 matrix legs pick their toolchain; the lint job sets no RUSTUP_TOOLCHAIN so rustfmt matches the pin)
  - Makefile:55-62 (musl build targets; not used by the new workflow)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't edit Makefile, cd.yml, nightly.yml or brew.yml here (later steps).

[PRE_GATE]
grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml

[MUTATION_SPEC]
1. .github/workflows/ci.yml: replace the whole file with exactly this content (2-space YAML indentation, LF line endings, trailing newline):

name: CI

on:
  push:
    branches: [master]
  pull_request:
    branches: [master]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: Build (${{ matrix.rust }})
    runs-on: ubuntu-latest
    strategy:
      fail-fast: false
      matrix:
        rust: [stable, "1.88"]
    env:
      RUSTUP_TOOLCHAIN: ${{ matrix.rust }}
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          components: clippy

      - name: Restore cargo cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: ci-${{ matrix.rust }}

      # gpgsm: needed by the x509 signing e2e test
      - name: Install commit-signing tools
        run: sudo apt-get update && sudo apt-get install -y gpgsm

      - name: Build Debug
        run: cargo build --workspace --locked

      - name: Run tests
        run: cargo test --workspace --locked

      - name: Run clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Build Release
        run: cargo build --release --locked

      - name: Binary Size
        run: ls -l ./target/release/gitui

  build-linux-musl:
    name: Build musl
    runs-on: ubuntu-latest
    env:
      RUSTUP_TOOLCHAIN: stable
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-musl

      - name: Restore cargo cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: ci-musl

      # gpgsm: needed by the x509 signing e2e test
      - name: Install musl and commit-signing tools
        run: sudo apt-get update && sudo apt-get install -y musl-tools gpgsm

      - name: Build Debug
        run: |
          cargo build --target x86_64-unknown-linux-musl --locked
          ./target/x86_64-unknown-linux-musl/debug/gitui --version

      - name: Build Release
        run: |
          cargo build --release --target x86_64-unknown-linux-musl --locked
          ./target/x86_64-unknown-linux-musl/release/gitui --version
          ls -l ./target/x86_64-unknown-linux-musl/release/gitui

      - name: Run tests
        run: cargo test --workspace --target x86_64-unknown-linux-musl --locked

  linting:
    name: Lints
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # no RUSTUP_TOOLCHAIN here: rustfmt must match rust-toolchain.toml
      - name: Install Rust
        uses: dtolnay/rust-toolchain@1.99.1
        with:
          components: rustfmt

      - name: Restore cargo cache
        uses: Swatinem/rust-cache@v2
        with:
          shared-key: ci-lint

      - name: rustfmt
        run: cargo fmt --all -- --check

      - name: tombi install
        uses: tombi-toml/setup-tombi@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          version: '0.9.0'

      - name: tombi check
        run: tombi format --check

      - name: cargo-deny install
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-deny@0.20.2

      - name: cargo-deny checks
        run: cargo deny check

   Why: removes the daily cron (P0-01), non-Linux/nightly legs (P0-02), `make clippy` (P0-03), the broken musl matrix/apt/target/install steps (P0-04), the five upstream-only jobs (P0-05), fmt without --all (P0-06), the unpinned source-built cargo-deny (P1-01), and the cache-key / `rustup override` drift (P2-01). The clippy and deny steps will stay red in CI until Phases 2–3 land; that is expected.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && { ! python3 -c 'import yaml' 2>/dev/null || python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/ci.yml"))'; } && ! grep -qE '^[[:space:]]+schedule:' .github/workflows/ci.yml && ! grep -qE 'macos-latest|windows-latest|nightly|armkeil|actions-rs/|rustup override|make clippy' .github/workflows/ci.yml && grep -qE 'rust: \[stable, "1\.88"\]' .github/workflows/ci.yml && grep -qF 'cargo clippy --workspace --all-targets -- -D warnings' .github/workflows/ci.yml && grep -qF 'cargo fmt --all -- --check' .github/workflows/ci.yml && grep -qF 'tool: cargo-deny@0.20.2' .github/workflows/ci.yml && grep -qF 'uses: dtolnay/rust-toolchain@1.99.1' .github/workflows/ci.yml && grep -qF 'sudo apt-get update && sudo apt-get install -y musl-tools gpgsm' .github/workflows/ci.yml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3A: `deny.toml` Advisory Scope and Dead Skip-Tree Entries

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3A: Make cargo-deny fail on transitive unsound advisories, add the reviewed heapless ignores, and drop the two skip-tree entries that match nothing, so `cargo deny check` passes with no unmatched-skip-root warnings.

[SCOPE_LOCK]
ALLOWED_FILES:
  - deny.toml
READ_ONLY_REFS:
  - deny.toml:17-21 ([advisories] with the RUSTSEC-2025-0141 ignore), deny.toml:26-28 (windows-sys entry + its 2 comment lines), deny.toml:36-39 (windows-core entry + its 3 comment lines)
  - heapless 0.8.0 reaches the build only through faster-hex 0.10.1 (`[target.'cfg(not(feature = "alloc"))'.dependencies.heapless]`); gix-hash 0.25.1 enables faster-hex `std` (implies `alloc`), so faster-hex never calls heapless. Advisories RUSTSEC-2026-0574 through RUSTSEC-2026-0582 (unsound, patched >= 0.9.4). Decision (Phase 2 review): reviewed ignore (BUG-0060).
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't change [licenses], [graph], [sources] or the other skip-tree entries.

[PRE_GATE]
grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml

[MUTATION_SPEC]
1. deny.toml [advisories]: add the line `unsound = "all"` directly above `ignore = [`. Why: cargo-deny's default unsound scope let heapless through on a transitive path (BUG-0060).
2. deny.toml [advisories] ignore: after the RUSTSEC-2025-0141 entry, add one entry per advisory RUSTSEC-2026-0574, -0575, -0576, -0577, -0578, -0579, -0580, -0581, -0582, each exactly in this form (id changes per line):
  { id = "RUSTSEC-2026-0574", reason = "heapless 0.8 via faster-hex 0.10; unreachable because gix-hash enables faster-hex std/alloc (BUG-0060)" },
   Put a comment line `# heapless 0.8.0: compiled but unused, see BUG-0060` above the first of them.
3. deny.toml [bans] skip-tree: delete `{ crate = "windows-sys" },` and its two comment lines above it, and delete `{ crate = "windows-core" },` and its three comment lines above it. Why: STEP 2D's Linux-only [graph] targets left them matching nothing (TASK-0024).
4. Run `cargo deny check`. If it now reports any other unsound advisory, report `Blocked` with the advisory id and `cargo tree -i <crate>` output. Don't add ignores beyond item 2.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo deny check && ! cargo deny check 2>&1 | grep -qF 'unmatched-skip-root' && grep -qE '^unsound = "all"$' deny.toml && test "$(grep -cE 'RUSTSEC-2026-05(7[4-9]|8[0-2])' deny.toml)" = 9 && ! grep -qE '"windows-(sys|core)"' deny.toml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3B: Allow unwrap/expect/panic in Tests

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3B: Turn on clippy's in-test allowances for unwrap, expect and panic, clearing the 1,312 test-only unwrap/expect diagnostics; filetreelist becomes fully clippy-clean.

[SCOPE_LOCK]
ALLOWED_FILES:
  - .clippy.toml
READ_ONLY_REFS:
  - .clippy.toml:1-2 (msrv = "1.88.0", cognitive-complexity-threshold = 18)
  - docs/baseline/clippy.txt (filetreelist's 64 errors are all unwrap in tests: filetreeitems.rs 52, filetree.rs 9, item.rs 3)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't add `#[allow]` attributes to source files; the crate-level `#![deny(clippy::unwrap_used, ...)]` lines stay as they are.

[PRE_GATE]
grep -qE '^channel = "1\.99\.1"$' rust-toolchain.toml

[MUTATION_SPEC]
1. .clippy.toml: append these three lines after the existing two:
allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
   Why: production code keeps the unwrap/expect/panic deny; only `#[test]` functions and `#[cfg(test)]` code get the allowance (audit P0-15, BUG-0012).

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo clippy -p filetreelist --all-targets -- -D warnings && grep -qE '^allow-unwrap-in-tests = true$' .clippy.toml && grep -qE '^allow-expect-in-tests = true$' .clippy.toml && grep -qE '^allow-panic-in-tests = true$' .clippy.toml

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3C: git2-hooks Clippy and Python Hook Tests

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3C: Fix git2-hooks' last clippy error and make both python hook tests run under python3, so git2-hooks is clippy-clean and test_pre_commit_fail_py really exercises its hook.

[SCOPE_LOCK]
ALLOWED_FILES:
  - git2-hooks/src/lib.rs
READ_ONLY_REFS:
  - git2-hooks/src/lib.rs:541 (`assert!(response.stderr.is_empty());`, clippy::assert_is_empty; `response.stderr` is a String)
  - git2-hooks/src/lib.rs:692-711 (test_pre_commit_py: unix hook starts `#!/usr/bin/env python`)
  - git2-hooks/src/lib.rs:713- (test_pre_commit_fail_py: same unix shebang; today it "passes" because `python` is missing and the hook exits 127)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep the `#[cfg(windows)]` hook bodies (`python.exe`) unchanged. No production code changes.

[PRE_GATE]
grep -qE '^allow-unwrap-in-tests = true$' .clippy.toml

[MUTATION_SPEC]
1. lib.rs:541: `assert!(response.stderr.is_empty());` → `assert_eq!(response.stderr, "");`.
2. lib.rs test_pre_commit_py, `#[cfg(not(windows))]` hook: first line `#!/usr/bin/env python` → `#!/usr/bin/env python3`.
3. lib.rs test_pre_commit_fail_py, `#[cfg(not(windows))]` hook: same change, `#!/usr/bin/env python` → `#!/usr/bin/env python3`. Why: on hosts with only python3 the hook exited 127, so the "fail" test passed without running its hook (BUG-0059).

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo clippy -p git2-hooks --all-targets -- -D warnings && cargo test -p git2-hooks --locked && ! grep -qE 'b"#!/usr/bin/env python$' git2-hooks/src/lib.rs && test "$(grep -cE 'b"#!/usr/bin/env python3$' git2-hooks/src/lib.rs)" = 2 && grep -qF 'assert_eq!(response.stderr, "");' git2-hooks/src/lib.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3D: asyncgit Clippy — blame, commit, commit_details

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3D: Clear the clippy errors in asyncgit's blame.rs, commit.rs and commit_details.rs test code.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/blame.rs
  - asyncgit/src/sync/commit.rs
  - asyncgit/src/sync/commit_details.rs
READ_ONLY_REFS:
  - blame.rs:196 and :226 (`file.write(b"line 2\n")?;`, `file.write(b"line 3\n")?;`; clippy::unused_io_amount)
  - commit.rs:190-191 (`#[cfg(test)]` / `mod tests {`), commit.rs:209 (`walk.read(&mut items).unwrap();`; clippy::read_zero_byte_vec)
  - commit_details.rs:202 (`fn test_msg_linefeeds() -> Result<()> {`) and :212 (`fn test_commit_message_combine() -> Result<()> {`); clippy::unnecessary_wraps
  - asyncgit/src/sync/logwalker.rs (`LogWalker::read(&mut self, out: &mut Vec<CommitId>)` is LogWalker's own method, not `std::io::Read`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No production code changes; don't rename LogWalker::read.

[PRE_GATE]
grep -qE '^allow-unwrap-in-tests = true$' .clippy.toml

[MUTATION_SPEC]
1. blame.rs:196 and :226: `file.write(` → `file.write_all(` (both lines).
2. commit.rs: on the line directly above `mod tests {` (after `#[cfg(test)]`), add
	// LogWalker::read fills a Vec<CommitId>; it is not io::Read, so this nursery lint is a false positive here
	#[allow(clippy::read_zero_byte_vec)]
   This is a workaround-shaped change: it silences read_zero_byte_vec for this test module only. It hides nothing real, because no io::Read is used on an empty Vec in these tests.
3. commit_details.rs test_msg_linefeeds and test_commit_message_combine: drop `-> Result<()>` from each signature and delete each function's trailing `Ok(())` line. If `Result` is then unused in the tests module, remove it from that module's `use` line.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && ! cargo clippy -p asyncgit --all-targets --message-format=short 2>&1 | grep -E '^asyncgit/src/sync/(blame|commit|commit_details)\.rs:' && ! grep -qE 'file\.write\(b"line' asyncgit/src/sync/blame.rs && grep -qF '#[allow(clippy::read_zero_byte_vec)]' asyncgit/src/sync/commit.rs && ! grep -qE 'fn test_(msg_linefeeds|commit_message_combine)\(\) -> Result' asyncgit/src/sync/commit_details.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3E: asyncgit Clippy — logwalker, sync/mod, merge_commit

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3E: Clear the clippy errors in asyncgit's logwalker.rs, sync/mod.rs and branch/merge_commit.rs test code.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/logwalker.rs
  - asyncgit/src/sync/mod.rs
  - asyncgit/src/sync/branch/merge_commit.rs
READ_ONLY_REFS:
  - logwalker.rs:179-180 (`#[cfg(test)]` / `mod tests {`); read_zero_byte_vec at :214, :239, :273, :318, :365; logwalker.rs:328 (`let diff_contains_bar = ...`, clippy::similar_names vs an earlier binding)
  - sync/mod.rs:115-116 (`#[cfg(test)]` / `pub mod tests {`); read_zero_byte_vec at :265; sync/mod.rs:319-323 (`let parents = if let Ok(id) = get_head_repo(&repo) { vec![repo.find_commit(id.into()).unwrap()] } else { Vec::new() };`, clippy::option_if_let_else)
  - branch/merge_commit.rs:189 (`let commits = get_commit_ids(&clone2, 10);`, similar_names vs `commit1` at :124)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No production code changes.

[PRE_GATE]
grep -qF '#[allow(clippy::read_zero_byte_vec)]' asyncgit/src/sync/commit.rs

[MUTATION_SPEC]
1. logwalker.rs and sync/mod.rs: add the same two lines as STEP 3D directly above each file's tests module line (`mod tests {` / `pub mod tests {`):
	// LogWalker::read fills a Vec<CommitId>; it is not io::Read, so this nursery lint is a false positive here
	#[allow(clippy::read_zero_byte_vec)]
2. logwalker.rs:328: rename the binding the lint names (run clippy to see the "existing binding" note) by appending `_filter` to the newer binding, e.g. `diff_contains_bar` → `diff_contains_bar_filter`, and update its uses in that test.
3. sync/mod.rs:319-323: replace the if-let/else with
		let parents = get_head_repo(&repo).map_or_else(
			|_| Vec::new(),
			|id| vec![repo.find_commit(id.into()).unwrap()],
		);
4. branch/merge_commit.rs:189: rename `commits` → `commit_ids` in that test and update every later use in the same test.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && ! cargo clippy -p asyncgit --all-targets --message-format=short 2>&1 | grep -E '^asyncgit/src/sync/(logwalker|mod|branch/merge_commit)\.rs:' && grep -qF '#[allow(clippy::read_zero_byte_vec)]' asyncgit/src/sync/logwalker.rs && grep -qF '#[allow(clippy::read_zero_byte_vec)]' asyncgit/src/sync/mod.rs && grep -qF 'get_head_repo(&repo).map_or_else(' asyncgit/src/sync/mod.rs && grep -qE '\blet commit_ids = get_commit_ids\(&clone2, 10\);' asyncgit/src/sync/branch/merge_commit.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3F: asyncgit Clippy — merge_ff, commits_info, hooks

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3F: Clear the clippy errors in asyncgit's branch/merge_ff.rs, commits_info.rs and hooks.rs test code.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/branch/merge_ff.rs
  - asyncgit/src/sync/commits_info.rs
  - asyncgit/src/sync/hooks.rs
READ_ONLY_REFS:
  - branch/merge_ff.rs:139 (`let commits = get_commit_ids(&clone1, 10);`, similar_names)
  - commits_info.rs:341-342 (`const FOREIGN_HASH: &str =` after statements, clippy::items_after_statements)
  - hooks.rs:388-396 (`if let HookResult::NotOk(res) = res { ... } else { assert!(false); }`, clippy::assertions_on_constants)
  - hooks.rs:438 (`const HOOKS_DIR: &str = "my_hooks";` after statements)
  - hooks.rs:485 (`git2_hooks::PrePushRef::to_stdin(&[update.clone()]);`, clippy::cloned_ref_to_slice_refs)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No production code changes.

[PRE_GATE]
grep -qF 'get_head_repo(&repo).map_or_else(' asyncgit/src/sync/mod.rs

[MUTATION_SPEC]
1. branch/merge_ff.rs:139: rename `commits` → `commit_ids` in that test and update every later use in the same test.
2. commits_info.rs: move the `const FOREIGN_HASH: &str = "d6d7d55cb6e4ba7301d6a11a657aab4211e5777e";` declaration to be the first statement of its test function.
3. hooks.rs:395: `assert!(false);` → `unreachable!("expected HookResult::NotOk");`.
4. hooks.rs:438: move `const HOOKS_DIR: &str = "my_hooks";` to be the first statement of its test function.
5. hooks.rs:485: `&[update.clone()]` → `std::slice::from_ref(&update)`.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && ! cargo clippy -p asyncgit --all-targets --message-format=short 2>&1 | grep -E '^asyncgit/src/sync/(branch/merge_ff|commits_info|hooks)\.rs:' && grep -qE '\blet commit_ids = get_commit_ids\(&clone1, 10\);' asyncgit/src/sync/branch/merge_ff.rs && ! grep -qF 'assert!(false)' asyncgit/src/sync/hooks.rs && grep -qF 'std::slice::from_ref(&update)' asyncgit/src/sync/hooks.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3G: asyncgit Clippy — cred, sign, stash

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3G: Clear the clippy errors in asyncgit's cred.rs, sign.rs and stash.rs test code.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/cred.rs
  - asyncgit/src/sync/sign.rs
  - asyncgit/src/sync/stash.rs
READ_ONLY_REFS:
  - cred.rs:308 and :361 (bare `#[should_panic]` on test_error_if_no_remote_when_trying_to_retrieve_if_need_username_password and test_error_if_no_remote_when_trying_to_extract_username_password; clippy::should_panic_without_expect)
  - sign.rs:695 (`.map(|field| field.to_string())`, clippy::redundant_closure_for_method_calls)
  - sign.rs:710-718 and :884-892 (`fn tool_available(...)` ending `.status().map(|s| s.success()).unwrap_or(false)`, clippy::map_unwrap_or)
  - stash.rs:149 (`assert!(get_stashes(repo_path).unwrap().is_empty());`, assert_is_empty; get_stashes returns Result<Vec<CommitId>>)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No production code changes. Don't touch the GNUPGHOME / SSH_AUTH_SOCK env handling in sign.rs (STEP 3I).

[PRE_GATE]
grep -qF 'std::slice::from_ref(&update)' asyncgit/src/sync/hooks.rs

[MUTATION_SPEC]
1. cred.rs:308 and :361: give each `#[should_panic]` the panic message it expects: run each test, read the panic message the code under test produces (`cargo test -p asyncgit --lib <test_name> -- --nocapture`), and write `#[should_panic(expected = "<a distinctive substring of that message>")]`.
2. sign.rs:695: `.map(|field| field.to_string())` → `.map(ToString::to_string)`.
3. sign.rs both `tool_available` functions: `.status()\n.map(|s| s.success())\n.unwrap_or(false)` → `.status()\n.is_ok_and(|s| s.success())`.
4. stash.rs:149: `assert!(get_stashes(repo_path).unwrap().is_empty());` → `assert_eq!(get_stashes(repo_path).unwrap(), Vec::new());`.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && ! cargo clippy -p asyncgit --all-targets --message-format=short 2>&1 | grep -E '^asyncgit/src/sync/(cred|sign|stash)\.rs:' && test "$(grep -cF '#[should_panic(expected = ' asyncgit/src/sync/cred.rs)" = 2 && ! grep -qE '^\s*#\[should_panic\]$' asyncgit/src/sync/cred.rs && test "$(grep -cF '.is_ok_and(|s| s.success())' asyncgit/src/sync/sign.rs)" = 2 && grep -qF '.map(ToString::to_string)' asyncgit/src/sync/sign.rs && grep -qF 'assert_eq!(get_stashes(repo_path).unwrap(), Vec::new());' asyncgit/src/sync/stash.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3H: asyncgit Clippy — remotes push and tags

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3H: Clear the last asyncgit clippy errors (remotes/push.rs, remotes/tags.rs) so `cargo clippy -p asyncgit --all-targets -- -D warnings` passes.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/remotes/push.rs
  - asyncgit/src/sync/remotes/tags.rs
READ_ONLY_REFS:
  - remotes/push.rs:300-301 (`#[test]` / `fn test_force_push_rewrites_history() {`, 109 lines; clippy::too_many_lines, limit 100)
  - remotes/tags.rs:294, :326, :366 (`assert!(tags_missing.is_empty());`; tags_missing is Vec<String>)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No production code changes.

[PRE_GATE]
grep -qF '.map(ToString::to_string)' asyncgit/src/sync/sign.rs

[MUTATION_SPEC]
1. remotes/push.rs: on the line between `#[test]` and `fn test_force_push_rewrites_history() {`, add
	#[allow(clippy::too_many_lines)] // one end-to-end two-clone scenario; splitting it hides the sequence
   This is workaround-shaped: it silences too_many_lines for this single test only.
2. remotes/tags.rs: each `assert!(tags_missing.is_empty());` (3 sites) → `assert_eq!(tags_missing, Vec::<String>::new());`.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo clippy -p asyncgit --all-targets -- -D warnings && grep -qF '#[allow(clippy::too_many_lines)]' asyncgit/src/sync/remotes/push.rs && test "$(grep -cF 'assert_eq!(tags_missing, Vec::<String>::new());' asyncgit/src/sync/remotes/tags.rs)" = 3

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3I: Stop Tests Mutating `GIT_DIR` / Leaking `GNUPGHOME`

* **Recommended Command:** `/boost`

```text
[STEP_ID]
STEP 3I: Run the GIT_DIR test in a child process so it no longer sets GIT_DIR for the whole test binary (the flaky test_commit race), and restore GNUPGHOME through a Drop guard in the two gpg e2e tests.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/revlog.rs
  - asyncgit/src/sync/sign.rs
READ_ONLY_REFS:
  - revlog.rs:372-403 (`#[test] #[serial] fn test_env_variables()`: `std::env::set_var("GIT_DIR", git_dir);` … `std::env::remove_var("GIT_DIR");` … `assert_eq!(result.unwrap(), ());`)
  - revlog.rs:325-340 (tests module imports: serial, TempDir, repo_init, RepoPath, AsyncLog, AsyncLogResult)
  - sign.rs:704 test_x509_sign_and_verify_e2e: `std::env::set_var("GNUPGHOME", home);` at :755, `std::env::remove_var("GNUPGHOME");` at :870
  - sign.rs:879 test_openpgp_sign_and_verify_e2e: set at :907, remove at :991
  - Root cause (BUG-0014): `#[serial]` only orders tests that also carry `#[serial]`. While test_env_variables runs, GIT_DIR is set for every concurrently running test, so their `Repository::open_ext` calls can open the wrong repo (baseline: sync::commit::tests::test_commit saw (0, 0) instead of (1, 0) once).
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No production code changes. Don't change the SSH_AUTH_SOCK handling in test_ssh_sign_and_verify_e2e.

[PRE_GATE]
grep -qF '#[allow(clippy::too_many_lines)]' asyncgit/src/sync/remotes/push.rs

[MUTATION_SPEC]
1. revlog.rs test_env_variables: keep its body, but make it run in a child copy of the test binary with GIT_DIR set only there. New shape:
	#[test]
	fn test_env_variables() {
		const CHILD: &str = "ASYNCGIT_TEST_ENV_VARIABLES_CHILD";
		if std::env::var_os(CHILD).is_none() {
			let (_td, repo) = repo_init().unwrap();
			let status = std::process::Command::new(
				std::env::current_exe().unwrap(),
			)
			.args(["--exact", "revlog::tests::test_env_variables"])
			.env(CHILD, "1")
			.env("GIT_DIR", repo.path())
			.status()
			.unwrap();
			assert!(status.success(), "child test failed: {status}");
			return;
		}
		// child process: GIT_DIR was set by the parent for this process only
		<existing body from `let (tx_git, _rx_git) = unbounded();` through `assert_eq!(result.unwrap(), ());`, minus the set_var/remove_var lines and minus the parent's repo_init/git_dir lines>
	}
   Remove `#[serial]` from it. Remove the `std::env::set_var("GIT_DIR", git_dir);` and `std::env::remove_var("GIT_DIR");` lines. If `serial` is then unused in revlog.rs's tests module, remove `use serial_test::serial;`.
2. sign.rs tests module: add a restore guard
	struct EnvRestore(&'static str, Option<std::ffi::OsString>);
	impl EnvRestore {
		fn set(key: &'static str, value: &std::path::Path) -> Self {
			let old = std::env::var_os(key);
			std::env::set_var(key, value);
			Self(key, old)
		}
	}
	impl Drop for EnvRestore {
		fn drop(&mut self) {
			match self.1.take() {
				Some(v) => std::env::set_var(self.0, v),
				None => std::env::remove_var(self.0),
			}
		}
	}
3. sign.rs both gpg e2e tests: replace `std::env::set_var("GNUPGHOME", home);` with `let _gnupghome = EnvRestore::set("GNUPGHOME", home);` and delete the matching `std::env::remove_var("GNUPGHOME");`. Why: a failing assert used to leave GNUPGHOME pointing at a deleted temp dir for the rest of the run. Both tests stay `#[serial]`.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo clippy -p asyncgit --all-targets -- -D warnings && cargo test -p asyncgit --locked && ! grep -qF 'set_var("GIT_DIR"' asyncgit/src/revlog.rs && grep -qF 'ASYNCGIT_TEST_ENV_VARIABLES_CHILD' asyncgit/src/revlog.rs && grep -qF '.env("GIT_DIR", repo.path())' asyncgit/src/revlog.rs && ! grep -qF 'set_var("GNUPGHOME"' asyncgit/src/sync/sign.rs && test "$(grep -cF 'EnvRestore::set("GNUPGHOME", home)' asyncgit/src/sync/sign.rs)" = 2 && grep -qF 'impl Drop for EnvRestore' asyncgit/src/sync/sign.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3J: gitui Clippy — app, options, diff

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3J: Clear the clippy errors in gitui's app.rs, options.rs and components/diff.rs test code.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/app.rs
  - src/options.rs
  - src/components/diff.rs
READ_ONLY_REFS:
  - src/app.rs:141-143 (test_env helper: `theme: Default::default(),`, `key_config: Default::default(),`, `repo: RefCell::new(RepoPath::Path(Default::default())),`; clippy::default_trait_access)
  - src/options.rs:40-41 (`repo: RefCell::new(RepoPath::Path(Default::default())),`, `data: Default::default(),`)
  - src/components/diff.rs:979 (`position: Default::default(),` in a DiffLine literal)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No production code changes.

[PRE_GATE]
grep -qE '^allow-unwrap-in-tests = true$' .clippy.toml

[MUTATION_SPEC]
1. Replace each `Default::default()` at the listed lines with the concrete type's `default()`, using the type clippy names in its message: app.rs:141 and :142 → `Rc::default()`, app.rs:143 → `PathBuf::default()`; options.rs:40 → `PathBuf::default()`, options.rs:41 → `OptionsData::default()`; diff.rs:979 → `DiffLinePosition::default()`. Add a `use` for any of these types the test module doesn't already import (`std::rc::Rc`, `std::path::PathBuf`, and the crate paths clippy prints).

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && ! cargo clippy -p gitui --all-targets --message-format=short 2>&1 | grep -E '^src/(app|options|components/diff)\.rs:' && grep -qF 'OptionsData::default()' src/options.rs && grep -qF 'DiffLinePosition::default()' src/components/diff.rs && grep -qF 'PathBuf::default()' src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 3K: gitui Clippy — reflow, key_config; Full Gate Green

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 3K: Clear the last gitui clippy errors (ui/reflow.rs, keys/key_config.rs) so the full workspace gate (clippy --all-targets -D warnings + cargo test --workspace) passes.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/ui/reflow.rs
  - src/keys/key_config.rs
READ_ONLY_REFS:
  - src/ui/reflow.rs:239 (`#[cfg(test)]`), the test-only `enum Composer` ending at :247 (`LineTruncator,`), `fn run_composer(which: Composer, ...)` at :249-251 (clippy::needless_pass_by_value), `let style = Default::default();` at :254, `width as u16` at :288, :293, :326, :329, :356, :361, :364 (clippy::cast_possible_truncation; `width` is usize)
  - src/keys/key_config.rs:148-149 (`#[test]` / `fn test_symbolic_links() {`, 102 lines; too_many_lines)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. No production code changes.

[PRE_GATE]
grep -qF 'OptionsData::default()' src/options.rs && grep -qF 'impl Drop for EnvRestore' asyncgit/src/sync/sign.rs && grep -qF 'assert_eq!(response.stderr, "");' git2-hooks/src/lib.rs

[MUTATION_SPEC]
1. reflow.rs: add `#[derive(Clone, Copy)]` on the line above the test-only `enum Composer {`.
2. reflow.rs:254: `let style = Default::default();` → `let style = Style::default();` (add `use ratatui::style::Style;` to the tests module if it isn't imported).
3. reflow.rs: every `width as u16` in the tests module → `u16::try_from(width).unwrap()`.
4. key_config.rs: on the line between `#[test]` and `fn test_symbolic_links() {`, add
	#[allow(clippy::too_many_lines)] // one end-to-end symlinked-config scenario
   This is workaround-shaped: it silences too_many_lines for this single test only.

[POST_GATE]
cargo fmt --all -- --check && cargo build --workspace --all-targets --locked && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'width as u16' src/ui/reflow.rs && grep -qF 'u16::try_from(width).unwrap()' src/ui/reflow.rs && grep -qF '#[allow(clippy::too_many_lines)]' src/keys/key_config.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

## Phase 1: Parallel Subagent Component Audit (completed)

STEP 1A was completed by Gemini; findings are in `AUDIT_REPORT.md`. It has no step block (nothing to execute).
