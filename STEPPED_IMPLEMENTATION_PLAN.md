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

**Base gate:** `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked`

**Forbidden paths:** `AUDIT_REPORT.md:CLAUDE.md:.orrery/*:scripts/*:wix/*:docs/*`

**Planning files:** `AUDIT_REPORT.md`

**Base gate note:** Phases 2–3 ran on a weaker gate (fmt + build) while HEAD couldn't pass clippy. Phase 3 made the full gate green (STEP 3M, 6c7855d3), so from Phase 4 on every POST_GATE starts with fmt + `clippy --workspace --all-targets -D warnings` + `cargo test --workspace --locked`.

**Gate environment:** POST_GATE runs twice: in the agent container (Neon/Hydrogen) and on the host (`phased:step_check`, authoritative). The container has no PyYAML, so YAML parse checks are written as `{ ! python3 -c 'import yaml' 2>/dev/null || <parse>; }`: skipped in the container, enforced on the host. The toolchain is pinned in `rust-toolchain.toml` (STEP 2J) so `cargo fmt` gives the same result in both places.

**How to read this:** work goes strictly top to bottom through **Up Next**, one step at a time. The **#** column is the only order. Step IDs are kept because commits and notes cite them. A step whose "Waits on" names an Open Question or a **[USER]** action doesn't start until that's resolved.

**Who runs what (phased workflow, through Orrery `gemini-phased`):**
- **Claude** writes the phase (full step blocks) and reviews it once at the end. It never builds or tests code here.
- **Gemini Hydrogen** (`agy-hydrogen-orch`, stage 0) routes the step; **Gemini Neon** (`agy-neon-impl`, stage 1) executes it; **Hydrogen** (`agy-hydrogen-val`, stage 2) reads the diff. Orrery's `phased:step_start` hook runs PRE_GATE, `neb start` and the snapshot; the `phased:step_check` gate runs `scripts/phased-step.sh check`; `phased:step_commit` commits on pass; `phased:step_defer` saves the attempt and restores the tree on deferral. Nothing failing is committed.
- **Neon** never commits, never edits this file, and never edits the forbidden paths. It files bugs and scope lapses as **pending** neb items.
- **Two failures** on one step → the step is deferred. A step whose PRE_GATE then fails is deferred without running. Too many deferrals (`max(2, ⌈steps/3⌉)`) stop the phase early.

**Phase 7 goal:** nothing in the UI stays stuck or shows the wrong state: tab index, loading lists, diff focus and selection, fetch/push availability, hints, error popups for tag/remote-URL failures, single-line inputs, and a fuzzy finder that Esc can cancel.

**Phase 7 pilot (7J–7M, approved 2026-11-21):** outcome steps. Each MUTATION_SPEC item is a required behavior and the coder chooses the implementation. The "Outcome steps" section of `.orrery/skills/repo-rules.md` overrides the exact-wording rules for these steps. 7B–7I are withdrawn.

_Phase 6 goal (done, reviewed 2026-11-21):_ popups get keys in draw order and don't leak them; panes, searches, highlighting, blame and fetch never stay blank, stuck or silent; file tree and stash list keep working selections and toggles.

_Phase 5 goal (done, reviewed 2026-11-21):_ no handler error exits the TUI, the audited panics are gone, and destructive actions hit the right target, start on the safe option and say what they delete.

_Phase 4 goal (done, reviewed 2026-11-21):_ closed the places where gitui was looser than git or unsafe with credentials, hooks, signing keys, editor launch, clipboard and its own log file (git-parity threat model).

_Phase 3 goal (done, reviewed 2026-11-21):_ clippy --workspace --all-targets -D warnings and cargo test --workspace pass reliably; cargo-deny fails on transitive unsound crates; Linux-only CI landed (2K).

_Phase 2 goal (done, reviewed 2026-11-21):_ the GitHub workflows are Linux-only and correct, `deny.toml` works with cargo-deny 0.20, `chrono` is replaced by `jiff`, `cargo deny check` passes, and the `Makefile` matches CI. Everything but 2A landed; 2A is re-queued as 2K.

**Side findings, bugs and scope lapses** become **pending neb items**, reviewed once at phase end. Every Up Next row has a neb task (`source_ref` = step ID); commits start with `[<TASK-ID>](<scope>)`.

### Up Next

| # | Step | Command | Scope | Target Files | Waits on |
| :---: | :--- | :---: | :--- | :--- | :--- |
| 1 | **STEP 7J** | `/goal` | Status Tab and Diff Behave Correctly | `src/components/*`, `src/tabs/status.rs`, `src/strings.rs` | — |
| 2 | **STEP 7K** | `/goal` | Keys and Hints Do What They Say | `src/components/textinput.rs`, `src/popups/*`, `src/tabs/revlog.rs`, `src/strings.rs`, `vim_style_key_config.ron` | — |
| 3 | **STEP 7L** | `/goal` | Failures and Confirmations Tell the Truth | `src/components/revision_files.rs`, `src/popups/*`, `src/strings.rs` | — |
| 4 | **STEP 7M** | `/goal` | Selections Survive a Refresh | `src/components/utils/*`, `src/components/commitlist.rs` | — |

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
| STEP 2K | ✅ ee00a44b | `/goal` | Linux-only CI workflow (re-queued 2A) | `.github/workflows/ci.yml` |
| STEP 3A | ✅ 258d4080 | `/goal` | deny.toml unsound scope, heapless ignores, dead skip-tree | `deny.toml` |
| STEP 3B | ✅ 1ecb26bc | `/goal` | Allow unwrap/expect/panic in tests | `.clippy.toml` |
| STEP 3C | ✅ 556a257d | `/goal` | git2-hooks clippy and python3 hook tests | `git2-hooks/src/lib.rs` |
| STEP 3J | ✅ 2d7ba5cd | `/goal` | gitui clippy: app, options, diff | `src/app.rs`, `src/options.rs`, `src/components/diff.rs` |
| STEP 3D | ✅ 7de34871 | `/goal` | asyncgit clippy: blame, commit, commit_details | `asyncgit/src/sync/{blame,commit,commit_details}.rs` |
| STEP 3E | ✅ 914a716c | `/goal` | asyncgit clippy: logwalker, sync/mod, merge_commit | `asyncgit/src/sync/logwalker.rs`, `asyncgit/src/sync/mod.rs`, `asyncgit/src/sync/branch/merge_commit.rs` |
| STEP 3F | ✅ 92b8df61 | `/goal` | asyncgit clippy: merge_ff, commits_info, hooks | `asyncgit/src/sync/branch/merge_ff.rs`, `asyncgit/src/sync/commits_info.rs`, `asyncgit/src/sync/hooks.rs` |
| STEP 3G | ✅ be414586 | `/goal` | asyncgit clippy: cred, sign, stash | `asyncgit/src/sync/{cred,sign,stash}.rs` |
| STEP 3H | ✅ 88ccd87b | `/goal` | asyncgit clippy: remotes push and tags | `asyncgit/src/sync/remotes/{push,tags}.rs` |
| STEP 3I | ↪ withdrawn | `/boost` | Deferred (agent image lacked gpg tools); re-queued as STEP 3L | `asyncgit/src/revlog.rs`, `asyncgit/src/sync/sign.rs` |
| STEP 3K | ↪ withdrawn | `/goal` | Deferred (PRE_GATE waited on 3I); re-queued as STEP 3M | `src/ui/reflow.rs`, `src/keys/key_config.rs` |
| STEP 3L | ✅ d9580191 | `/boost` | GIT_DIR test in a child process, GNUPGHOME guard (re-queued 3I) | `asyncgit/src/revlog.rs`, `asyncgit/src/sync/sign.rs` |
| STEP 3M | ✅ 6c7855d3 | `/goal` | gitui clippy: reflow, key_config; full gate green (re-queued 3K) | `src/ui/reflow.rs`, `src/keys/key_config.rs` |
| STEP 4A | ✅ 90a3db8d | `/goal` | Credential URL checks | `asyncgit/src/sync/cred.rs` |
| STEP 4B | ✅ 60bf599e | `/goal` | fetch_all keeps credentials on their host | `asyncgit/src/sync/remotes/mod.rs` |
| STEP 4D | ✅ 4b8e56a1 | `/goal` | Hook path: ~ only, regular files, `--` | `git2-hooks/src/hookspath.rs` |
| STEP 4E | ✅ ffe74ceb | `/goal` | SSH key:: literals of any type | `asyncgit/src/sync/sign.rs` |
| STEP 4F | ✅ 80c47289 | `/goal` | No writes through symlinks on discard | `asyncgit/src/sync/utils.rs` |
| STEP 4G | ✅ 6e10b62e | `/boost` | Editor via sh -c, exit status, no drop-guard panic | `src/popups/externaleditor.rs` |
| STEP 4H | ✅ 1b5a5b07 | `/goal` | Clipboard lookup and OSC 52 flush | `src/clipboard.rs` |
| STEP 4I | ✅ 08f7022b | `/goal` | Trace log 0600, no symlink | `src/args.rs` |
| STEP 4J | ✅ f7995523 | `/goal` | Credential prompt clears both inputs | `src/components/cred.rs` |
| STEP 4K | ✅ 8dc7eadb | `/goal` | commit.template expands ~ only | `src/popups/commit.rs` |
| STEP 4C | ✅ 8d452264 | `/goal` | Redact passwords in Debug and logs | `asyncgit/src/sync/cred.rs`, `asyncgit/src/sync/remotes/callbacks.rs` |
| STEP 5A | ✅ 7cd02752 | `/boost` | Error boundaries in App | `src/app.rs` |
| STEP 5J | ✅ f5d20cfd | `/goal` | Reset/checkout popups open on the safe option | `src/popups/reset.rs`, `src/popups/checkout_option.rs` |
| STEP 5K | ✅ 3f79314d | `/goal` | File discard prompt names the path | `src/popups/confirm.rs`, `src/strings.rs` |
| STEP 5L | ✅ 0cb2aa18 | `/goal` | Tighten an existing trace log to 0600 | `src/args.rs` |
| STEP 5B | ✅ 10c7fb9c | `/boost` | Delete the right remote branch | `src/app.rs`, `src/popups/push.rs` |
| STEP 5C | ✅ 2f3d4d0a | `/goal` | Tag delete without remote, unstage-all errors | `src/app.rs`, `src/components/changes.rs` |
| STEP 5D | ✅ b6c9f071 | `/goal` | Tree underflow panics | `filetreelist/src/filetreeitems.rs`, `src/components/utils/statustree.rs` |
| STEP 5E | ✅ 682bb6a0 | `/goal` | Commit list and status tree panics | `src/components/commitlist.rs`, `src/components/status_tree.rs`, `src/components/utils/filetree.rs` |
| STEP 5F | ✅ 6f02c395 | `/goal` | Tag, blame, remote list panics | `src/popups/taglist.rs`, `src/popups/blame_file.rs`, `src/popups/remotelist.rs` |
| STEP 5G | ✅ aa9ee1ec | `/goal` | Branch list status error | `src/popups/branchlist.rs` |
| STEP 5H | ✅ f1d5782d | `/goal` | Cancel credentials cancels push/push-tags | `src/popups/push.rs`, `src/popups/push_tags.rs` |
| STEP 5I | ✅ e6ec537b | `/goal` | Cancel and preflight errors in pull/fetch | `src/popups/pull.rs`, `src/popups/fetch.rs` |
| STEP 6A | ✅ 4886b671 | `/goal` | Popup event order = reverse draw order | `src/app.rs` |
| STEP 6D | ✅ 63a59df4 | `/goal` | Panes come back after blame/history | `src/tabs/files.rs`, `src/components/changes.rs`, `src/tabs/stashing.rs` |
| STEP 6F | ✅ e50b2d2d | `/goal` | File tree keeps a valid selection | `filetreelist/src/filetree.rs`, `filetreelist/src/filetreeitems.rs` |
| STEP 6H | ✅ 34faeed8 | `/goal` | REMOTES bit, branch list refresh, truncation | `src/queue.rs`, `src/popups/branchlist.rs`, `src/popups/submodules.rs` |
| STEP 6I | ✅ 8f8fe527 | `/goal` | Show fetch errors | `asyncgit/src/fetch_job.rs`, `src/popups/fetch.rs` |
| STEP 6J | ✅ b9e2efe9 | `/goal` | Blame errors stop the spinner | `asyncgit/src/blame.rs`, `src/popups/blame_file.rs` |
| STEP 6B | ✅ 352989c7 | `/goal` | Modal popups consume their keys | `src/popups/revision_files.rs`, `src/popups/taglist.rs`, `src/popups/inspect_commit.rs` |
| STEP 6C | ✅ 62d80cf9 | `/goal` | Stash list keys, fuzzy nav, tab-switch refresh | `src/tabs/stashlist.rs`, `src/popups/fuzzy_find.rs`, `src/app.rs` |
| STEP 6E | ✅ 433f8465 | `/goal` | Search/highlight finish while tab hidden | `src/tabs/revlog.rs`, `src/tabs/files.rs` |
| STEP 6G | ✅ 6c1f2f14 | `/goal` | Stashing tab honors the untracked toggle | `src/tabs/stashing.rs` |
| STEP 7A | ✅ b4eecc8c | `/goal` | Bad tab index, Typechange symbol, key symbol errors | `src/options.rs`, `src/components/status_tree.rs`, `src/keys/symbols.rs` |
| STEP 7B | ↪ withdrawn | `/goal` | Deferred (spec defect TASK-0082); replaced by STEP 7J | `src/components/diff.rs` |
| STEP 7D | ↪ withdrawn | `/goal` | Not run (pilot re-plan); replaced by STEP 7K and 7L | `src/tabs/revlog.rs`, `src/strings.rs`, `src/popups/confirm.rs` |
| STEP 7E | ↪ withdrawn | `/goal` | Not run (pilot re-plan); replaced by STEP 7K | `src/components/textinput.rs`, `vim_style_key_config.ron` |
| STEP 7F | ↪ withdrawn | `/goal` | Not run (pilot re-plan); replaced by STEP 7L | `src/components/revision_files.rs` |
| STEP 7G | ↪ withdrawn | `/goal` | Not run (pilot re-plan); replaced by STEP 7M | `src/components/utils/statustree.rs`, `src/components/commitlist.rs` |
| STEP 7H | ↪ withdrawn | `/goal` | Not run (pilot re-plan); replaced by STEP 7L | `src/popups/taglist.rs`, `src/popups/remotelist.rs` |
| STEP 7I | ↪ withdrawn | `/goal` | Not run (pilot re-plan); replaced by STEP 7K | `src/popups/fuzzy_find.rs` |
| STEP 7C | ↪ withdrawn | `/goal` | Not run (pilot re-plan); replaced by STEP 7J | `src/tabs/status.rs` |

### User Step: IMAGE

The Orrery agent image now installs `gnupg gpgsm openssh-client python3-yaml` (Dockerfile at `/srv/docker-llms/orrery/cli-agents/Dockerfile`). Rebuild it under the same tag, then check the tools:

```bash
rm -rf /tmp/nebsrc && mkdir -p /tmp/nebsrc && git -C /srv/devshare/Nebulous archive HEAD | tar -x -C /tmp/nebsrc
```

```bash
docker build --build-context agybin="$HOME/.gemini/bin" --build-context geminicfg="$HOME/.gemini/config" --build-context nebsrc=/tmp/nebsrc -t orrery-cli-agents:cc2.1.318-agy1.5.13-rs1.99.1-neb /srv/docker-llms/orrery/cli-agents
```

```bash
docker run --rm --entrypoint bash orrery-cli-agents:cc2.1.318-agy1.5.13-rs1.99.1-neb -c 'command -v gpg gpgsm ssh-keygen && python3 -c "import yaml" && echo IMAGE_OK'
```

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

- **Phase 8 — Confirmations, hints, layout, input validation.** P2-08 (confirm dialogs for revert/merge/rebase/detached checkout/submodule update), P2-12 (key-hint mismatches), P2-16 (80x24 details layout, scroll clamping), P2-23 (pre-submit ref/input validation), the remaining P2-20 (commit-list marks across batches, display widths) and P2-22 (unborn-HEAD Files tab, RevisionFiles focus/`e` behind popup) parts.
- **Phase 9 — Amber CRT theme and chrome.** Goal: the Locked Decisions' styling tokens are the default theme, with `theme.ron` overrides still working.
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

### STEP 3L: Stop Tests Mutating `GIT_DIR` / Leaking `GNUPGHOME`

* **Recommended Command:** `/boost`

Re-queue of STEP 3I (deferred in Phase 3: the agent image had no gpg/gpgsm/ssh-keygen, so asyncgit's signing e2e tests failed in the container; the image now installs them, see User Step: IMAGE). Block otherwise unchanged.

```text
[STEP_ID]
STEP 3L: Run the GIT_DIR test in a child process so it no longer sets GIT_DIR for the whole test binary (the flaky test_commit race), and restore GNUPGHOME through a Drop guard in the two gpg e2e tests.

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

### STEP 3M: gitui Clippy — reflow, key_config; Full Gate Green

* **Recommended Command:** `/goal`

Re-queue of STEP 3K (deferred in Phase 3: the agent image had no gpg/gpgsm/ssh-keygen, so asyncgit's signing e2e tests failed in the container; the image now installs them, see User Step: IMAGE). Block otherwise unchanged.

```text
[STEP_ID]
STEP 3M: Clear the last gitui clippy errors (ui/reflow.rs, keys/key_config.rs) so the full workspace gate (clippy --all-targets -D warnings + cargo test --workspace) passes.

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
## Phase 4: Security hardening (git-parity threat model)

**Threat model (Phase 3 review decision):** match git. Repo-local `.git/config` is trusted the way git trusts it; libgit2 already refuses to open repos the user doesn't own (`safe.directory`), and asyncgit keeps that check on (`Repository::open_ext(.., FROM_ENV, ..)` in `asyncgit/src/sync/repository.rs:56-60`). Phase 4 fixes the places where gitui is looser than git or plainly buggy. The audit's "ignore repo-local config" fixes (P1-12, P1-13, and those parts of P1-02/P1-21) are parked, not done.

### STEP 4A: Credential URL Checks

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4A: Make asyncgit's credential lookup treat only http/https URLs as needing a password, use the push URL when extracting push credentials, and refuse remote URLs containing control characters before they reach the credential helper.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/cred.rs
READ_ONLY_REFS:
  - asyncgit/src/sync/cred.rs:38-87 (need_username_password*: `let is_http = url.starts_with("http");` three times)
  - asyncgit/src/sync/cred.rs:89-176 (extract_username_password, _for_fetch, _for_push: each builds `url` then `CredentialHelper::new(&url)`; _for_push reads `.url()` only, unlike need_username_password_for_push at :72-87 which prefers `.pushurl()`)
  - asyncgit/src/error.rs:86 (`Generic(String)` variant)
  - git's CVE-2020-5260: a newline in the URL lets a remote inject extra key=value lines into the credential-helper protocol
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep every pub fn signature. Keep reading `credential.*` from `repo.config()` (git-parity: repo-local config is trusted).

[PRE_GATE]
true

[MUTATION_SPEC]
1. cred.rs: add two private helpers above `need_username_password`:
	/// true only for real http(s) URLs (`starts_with("http")` also matched `httpfoo://`)
	fn is_http_url(url: &str) -> bool {
		url::Url::parse(url)
			.is_ok_and(|u| matches!(u.scheme(), "http" | "https"))
	}

	/// a newline or other control character in the URL could inject
	/// lines into the credential-helper protocol (CVE-2020-5260 class)
	fn ensure_no_control_chars(url: &str) -> Result<()> {
		if url.chars().any(char::is_control) {
			return Err(Error::Generic(String::from(
				"remote url contains control characters",
			)));
		}
		Ok(())
	}
2. cred.rs: in all three need_username_password* functions, `let is_http = url.starts_with("http");` → `let is_http = is_http_url(&url);`.
3. cred.rs: in all three extract_username_password* functions, insert `ensure_no_control_chars(&url)?;` on the line before `let mut helper = CredentialHelper::new(&url);`.
4. cred.rs extract_username_password_for_push: build `url` the same way need_username_password_for_push does (prefer `remote.pushurl()`, fall back to `remote.url()`, else `Error::UnknownRemote`), so credentials are looked up for the URL the push actually goes to.
5. Tests, in cred.rs's tests module:
	#[test]
	fn test_is_http_url() {
		assert!(is_http_url("https://example.com/repo.git"));
		assert!(is_http_url("http://example.com/repo.git"));
		assert!(!is_http_url("httpfoo://example.com/repo.git"));
		assert!(!is_http_url("git@example.com:repo.git"));
		assert!(!is_http_url("ssh://git@example.com/repo.git"));
	}
	#[test]
	fn test_ensure_no_control_chars() {
		assert!(ensure_no_control_chars("https://example.com/repo.git").is_ok());
		assert!(ensure_no_control_chars("https://example.com/r\nhost=evil.example.com").is_err());
		assert!(ensure_no_control_chars("https://example.com/r\r").is_err());
	}
   cred.rs's tests module does not glob-import `super::*`; add `use super::{ensure_no_control_chars, is_http_url};` to its imports.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'starts_with("http")' asyncgit/src/sync/cred.rs && test "$(grep -cF 'ensure_no_control_chars(&url)?;' asyncgit/src/sync/cred.rs)" = 3 && test "$(grep -cF 'is_http_url(&url)' asyncgit/src/sync/cred.rs)" = 3 && grep -qE '\bfn test_ensure_no_control_chars\b' asyncgit/src/sync/cred.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4B: Don't Send One Remote's Credentials to Every Remote

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4B: In fetch_all, pass the user's credentials only to remotes on the same scheme/host/port as the default remote they were entered for; other remotes get none.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/remotes/mod.rs
READ_ONLY_REFS:
  - asyncgit/src/sync/remotes/mod.rs:301-334 (`pub fn fetch_all(repo_path, basic_credential, progress_sender)`: loops over all remotes calling `fetch_from_remote(repo_path, &remote, basic_credential.clone(), None)?`)
  - asyncgit/src/sync/remotes/mod.rs:239 (`pub(crate) fn get_default_remote_in_repo(repo: &Repository) -> Result<String>`)
  - src/popups/fetch.rs:64-66 (the credentials come from `extract_username_password`, i.e. the default remote)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep fetch_all's signature.

[PRE_GATE]
true

[MUTATION_SPEC]
1. remotes/mod.rs: add a private helper above fetch_all:
	/// scheme, host and port of a remote's URL, used to keep credentials on the host they were entered for
	fn remote_url_origin(
		repo: &git2::Repository,
		name: &str,
	) -> Option<(String, String, Option<u16>)> {
		let remote = repo.find_remote(name).ok()?;
		let url = url::Url::parse(remote.url()?).ok()?;
		Some((
			url.scheme().to_owned(),
			url.host_str()?.to_owned(),
			url.port_or_known_default(),
		))
	}
2. remotes/mod.rs fetch_all: after `let remotes_count = remotes.len();`, add
	let cred_origin = get_default_remote_in_repo(&repo)
		.ok()
		.and_then(|name| remote_url_origin(&repo, &name));
   and in the loop pass
	let cred = if cred_origin.is_some()
		&& remote_url_origin(&repo, &remote) == cred_origin
	{
		basic_credential.clone()
	} else {
		None
	};
   to fetch_from_remote instead of `basic_credential.clone()`.
3. Test, in remotes/mod.rs's tests module:
	#[test]
	fn test_remote_url_origin() {
		let (_td, repo) = repo_init().unwrap();
		repo.remote("origin", "https://example.com/a.git").unwrap();
		repo.remote("same", "https://example.com:443/c.git").unwrap();
		repo.remote("other", "https://example.org/b.git").unwrap();
		repo.remote("ssh", "git@example.com:a.git").unwrap();
		let origin = remote_url_origin(&repo, "origin");
		assert!(origin.is_some());
		assert_eq!(remote_url_origin(&repo, "same"), origin);
		assert_ne!(remote_url_origin(&repo, "other"), origin);
		assert_eq!(remote_url_origin(&repo, "ssh"), None);
	}
   Use the tests module's existing `repo_init` import; add `use super::remote_url_origin;` if the module doesn't glob-import `super::*`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qE '\bfn remote_url_origin\(' asyncgit/src/sync/remotes/mod.rs && grep -qF 'remote_url_origin(&repo, &remote) == cred_origin' asyncgit/src/sync/remotes/mod.rs && grep -qE '\bfn test_remote_url_origin\b' asyncgit/src/sync/remotes/mod.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4C: Redact Passwords from Debug Output and Logs

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4C: Stop BasicAuthCredential's Debug output and the credentials callback's debug log from printing passwords or URL userinfo (gitui -l writes a trace log).

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/cred.rs
  - asyncgit/src/sync/remotes/callbacks.rs
READ_ONLY_REFS:
  - asyncgit/src/sync/cred.rs:15-22 (`#[derive(Debug, Clone, Default, PartialEq, Eq)] pub struct BasicAuthCredential { pub username: Option<String>, pub password: Option<String> }`)
  - asyncgit/src/sync/remotes/callbacks.rs:179-187 (`fn credentials(&self, url: &str, ...)` then `log::debug!("creds: '{url}' {username_from_url:?} ({allowed_types:?})",);`)
  - PushRequest/FetchRequest/PushTagsRequest derive Debug and contain BasicAuthCredential, so the manual Debug covers them.
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep BasicAuthCredential's fields, Clone/Default/PartialEq/Eq derives and pub API.

[PRE_GATE]
grep -qF 'ensure_no_control_chars(&url)?;' asyncgit/src/sync/cred.rs

[MUTATION_SPEC]
1. cred.rs: remove `Debug` from BasicAuthCredential's derive list and add
	impl std::fmt::Debug for BasicAuthCredential {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			f.debug_struct("BasicAuthCredential")
				.field("username", &self.username)
				.field(
					"password",
					&self.password.as_ref().map(|_| "<redacted>"),
				)
				.finish()
		}
	}
2. callbacks.rs credentials: replace the `log::debug!("creds: ...")` call with
		let shown_url = url::Url::parse(url).map_or_else(
			|_| String::from("<unparsable url>"),
			|mut u| {
				let _ = u.set_password(None);
				u.to_string()
			},
		);
		log::debug!(
			"creds: '{shown_url}' {username_from_url:?} ({allowed_types:?})",
		);
3. Test, in cred.rs's tests module:
	#[test]
	fn test_debug_redacts_password() {
		let cred = BasicAuthCredential::new(
			Some(String::from("testuser")),
			Some(String::from("s3cret-value")),
		);
		let shown = format!("{cred:?}");
		assert!(shown.contains("testuser"));
		assert!(!shown.contains("s3cret-value"));
		assert!(shown.contains("<redacted>"));
	}

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'impl std::fmt::Debug for BasicAuthCredential' asyncgit/src/sync/cred.rs && ! grep -qF "\"creds: '{url}'" asyncgit/src/sync/remotes/callbacks.rs && grep -qF 'u.set_password(None)' asyncgit/src/sync/remotes/callbacks.rs && grep -qE '\bfn test_debug_redacts_password\b' asyncgit/src/sync/cred.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4D: Hook Path Expansion and Executable Check

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4D: Expand only `~` in core.hooksPath (as git does, not $VARS), treat only regular files as executable hooks, and pass `--` before the hook path in the sh fallback.

[SCOPE_LOCK]
ALLOWED_FILES:
  - git2-hooks/src/hookspath.rs
READ_ONLY_REFS:
  - git2-hooks/src/hookspath.rs:62-70 (`fn expand_path`: `let hook_expanded = shellexpand::full(path.as_os_str().to_str().ok_or(HooksError::PathToString)?,)?;`)
  - git2-hooks/src/hookspath.rs:250-256 (`match run_command(&mut Command::new(&hook)) { Err(err) if err.raw_os_error() == Some(ENOEXEC) => { run_command(sh_command().arg(&hook)) } ...`)
  - git2-hooks/src/hookspath.rs:294-308 (`fn is_executable`: `permissions.mode() & 0o111 != 0`; a directory with mode 0755 counts as executable)
  - git2-hooks/src/hookspath.rs:315- (test module: test_hookspath_relative, test_hookspath_absolute)
  - `shellexpand::tilde(&str) -> Cow<str>` (infallible)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep the Windows `sh -c` branch and the `HooksError::ShellExpand` variant (error.rs) unchanged.

[PRE_GATE]
true

[MUTATION_SPEC]
1. hookspath.rs expand_path: `shellexpand::full(...)?` → `shellexpand::tilde(...)` (drop the trailing `?`; the `.ok_or(HooksError::PathToString)?` on the str conversion stays). Why: git expands only `~` in core.hooksPath; `$VAR` expansion let config pull environment values into the hook path.
2. hookspath.rs unix ENOEXEC fallback: `run_command(sh_command().arg(&hook))` → `run_command(sh_command().arg("--").arg(&hook))`.
3. hookspath.rs unix is_executable: return `metadata.is_file() && permissions.mode() & 0o111 != 0` (a directory named like a hook is not a hook).
4. Tests, in hookspath.rs's test module:
	#[test]
	fn test_hookspath_does_not_expand_env_vars() {
		assert_eq!(
			HookPaths::expand_path(
				Path::new("$HOME/hooks/pre-commit"),
				Path::new("example_git_root"),
			)
			.unwrap(),
			Path::new("example_git_root").join("$HOME/hooks/pre-commit")
		);
	}
	#[cfg(unix)]
	#[test]
	fn test_directory_is_not_executable() {
		let dir = tempfile::tempdir().unwrap();
		assert!(!super::is_executable(dir.path()));
	}
   If `tempfile` is not a git2-hooks dev-dependency, write the directory test with `std::env::temp_dir()` instead (an existing directory).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'shellexpand::full' git2-hooks/src/hookspath.rs && grep -qF 'shellexpand::tilde(' git2-hooks/src/hookspath.rs && grep -qF 'sh_command().arg("--").arg(&hook)' git2-hooks/src/hookspath.rs && grep -qF 'metadata.is_file()' git2-hooks/src/hookspath.rs && grep -qE '\bfn test_hookspath_does_not_expand_env_vars\b' git2-hooks/src/hookspath.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4E: SSH `key::` Literals of Any Key Type

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4E: Treat every `key::<literal>` user.signingKey as an inline key written to a temp file (as git does), not only literals starting with `ssh-`; today `key::ecdsa-sha2-…` or `key::sk-ssh-…` is passed to `ssh-keygen -f` as if it were a file path.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/sign.rs
READ_ONLY_REFS:
  - asyncgit/src/sync/sign.rs:193-234 (ssh branch of SignBuilder::from_gitconfig: strips `key::`, then `if signing_key.starts_with("ssh-") { write_signing_key_to_temp_file .. } else { (signing_key, None) }`)
  - asyncgit/src/sync/sign.rs:521-539 (test_ssh_keyliteral_config: `config.set_str("user.signingKey", "ssh-ed25519 test")`, asserts `PathBuf::from(sign.signing_key()).is_file()`)
  - git-config(1) user.signingKey: "If gpg.format is set to ssh this can contain the path to either your private ssh key or the public key ... Alternatively it can contain a public key prefixed with key:: directly"; git also still accepts a bare `ssh-` literal.
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't change the GPG/x509 branches or the e2e tests.

[PRE_GATE]
true

[MUTATION_SPEC]
1. sign.rs ssh branch: replace the strip + `starts_with("ssh-")` logic with
				// `key::<literal>` is git's syntax for an inline key; a bare
				// `ssh-…` value is also treated as a literal (legacy git behavior)
				let (literal, signing_key) =
					match signing_key.strip_prefix("key::") {
						Some(rest) => (true, rest.to_string()),
						None => (
							signing_key.starts_with("ssh-"),
							signing_key,
						),
					};
   and use `if literal {` where the code had `if signing_key.starts_with("ssh-") {`.
2. Test, next to test_ssh_keyliteral_config:
	#[test]
	fn test_ssh_key_prefixed_literal_any_type() -> Result<()> {
		use std::path::PathBuf;
		let (_tmp_dir, repo) = repo_init_empty()?;
		{
			let mut config = repo.config()?;
			config.set_str("gpg.format", "ssh")?;
			config.set_str("user.signingKey", "key::ecdsa-sha2-nistp256 test")?;
		}
		let sign = SignBuilder::from_gitconfig(&repo, &repo.config()?)?;
		let path = PathBuf::from(sign.signing_key());
		assert!(path.is_file());
		assert_eq!(std::fs::read_to_string(path)?.trim_end(), "ecdsa-sha2-nistp256 test");
		Ok(())
	}

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'match signing_key.strip_prefix("key::")' asyncgit/src/sync/sign.rs && ! grep -qF 'if signing_key.starts_with("ssh-") {' asyncgit/src/sync/sign.rs && grep -qE '\bfn test_ssh_key_prefixed_literal_any_type\b' asyncgit/src/sync/sign.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4F: Don't Write Through Symlinks When Discarding Lines

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4F: Make repo_write_file refuse to write through a symlink, so discarding lines of a symlinked file can't overwrite the file it points to (possibly outside the repo).

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/sync/utils.rs
READ_ONLY_REFS:
  - asyncgit/src/sync/utils.rs:181-193 (`pub(crate) fn repo_write_file(repo, file, content)`: `File::create(work_dir(repo)?.join(file))` then `write_all`)
  - asyncgit/src/sync/staging/discard_tracked.rs:40 (the one production caller: `repo_write_file(&repo, file_path, new_content.as_str())?;`)
  - asyncgit/src/error.rs:86 (`Generic(String)`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep the signature; all existing test callers write regular files and must keep working.

[PRE_GATE]
true

[MUTATION_SPEC]
1. utils.rs repo_write_file: before `File::create`, add
	if std::fs::symlink_metadata(file_path)
		.is_ok_and(|m| m.file_type().is_symlink())
	{
		return Err(Error::Generic(format!(
			"refusing to write through symlink: {file_path}"
		)));
	}
2. Test, in utils.rs's tests module:
	#[cfg(unix)]
	#[test]
	fn test_repo_write_file_refuses_symlink() {
		let (_td, repo) = repo_init().unwrap();
		let root = repo.workdir().unwrap();
		let outside = tempfile::NamedTempFile::new().unwrap();
		std::fs::write(outside.path(), "keep").unwrap();
		std::os::unix::fs::symlink(outside.path(), root.join("link.txt")).unwrap();
		assert!(repo_write_file(&repo, "link.txt", "overwritten").is_err());
		assert_eq!(std::fs::read_to_string(outside.path()).unwrap(), "keep");
	}

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'refusing to write through symlink' asyncgit/src/sync/utils.rs && grep -qE '\bfn test_repo_write_file_refuses_symlink\b' asyncgit/src/sync/utils.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4G: Run the Editor the Way git Does

* **Recommended Command:** `/boost`

```text
[STEP_ID]
STEP 4G: Launch the external editor through `sh -c '<editor> "$@"'` like git (so quoted editor commands work), treat a non-zero editor exit as an error (`:cq` aborts), and stop the terminal-restore guard from panicking.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/externaleditor.rs
READ_ONLY_REFS:
  - src/popups/externaleditor.rs:47-115 (`pub fn open_file_in_editor(repo, path)`: `defer! { io::stdout().execute(EnterAlternateScreen).expect("reset terminal"); }`, editor lookup GIT_EDITOR → core.editor → VISUAL → EDITOR → "vi", hand-rolled split on the first `"`/space, then `Command::new(command).current_dir(work_dir).args(args).status()` ignoring the exit status)
  - git's run-command.c prepare_shell_cmd / editor.c launch_specified_editor: the editor string is run as `sh -c '<editor> "$@"' '<editor>' <file>`; a non-zero exit makes git report "There was a problem with the editor" and abort.
  - Callers: src/popups/commit.rs (commit message editing) treats Err as "don't use the edited message".
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep the editor lookup order (GIT_EDITOR, core.editor, VISUAL, EDITOR, vi) and the empty-editor error.

[PRE_GATE]
true

[MUTATION_SPEC]
1. defer! block: `io::stdout().execute(EnterAlternateScreen).expect("reset terminal");` → `if let Err(e) = io::stdout().execute(EnterAlternateScreen) { log::error!("reset terminal failed: {e}"); }`.
2. Replace the hand-rolled splitting (from the `// TODO: proper handling arguments containing whitespaces` comment through the `Command::new(command.clone())...` call) with, under `#[cfg(unix)]`:
		if editor.trim().is_empty() {
			bail!("editor env variable found empty: {}", environment_options.join(" or "));
		}
		let status = Command::new("sh")
			.current_dir(&work_dir)
			.arg("-c")
			.arg(format!("{editor} \"$@\""))
			.arg(&editor)
			.arg(&path)
			.status()
			.map_err(|e| anyhow!("\"{editor}\": {e}"))?;
		if !status.success() {
			bail!("editor \"{editor}\" exited with {status}");
		}
   and keep the existing splitting code under `#[cfg(not(unix))]` (still followed by the same `status.success()` check). Remove imports that become unused on unix.
3. Tests, in externaleditor.rs (add `#[cfg(test)] mod tests` if absent; unix only). Factor the launch into `fn run_editor(editor: &str, work_dir: &Path, path: &Path) -> Result<()>` holding item 2's unix code, call it from open_file_in_editor, and test it:
	#[cfg(unix)]
	#[test]
	fn test_run_editor_quoted_command() {
		let dir = tempfile::tempdir().unwrap();
		let file = dir.path().join("msg.txt");
		std::fs::write(&file, "").unwrap();
		run_editor("sh -c 'printf edited > \"$1\"' sh", dir.path(), &file).unwrap();
		assert_eq!(std::fs::read_to_string(&file).unwrap(), "edited");
	}
	#[cfg(unix)]
	#[test]
	fn test_run_editor_nonzero_exit_is_error() {
		let dir = tempfile::tempdir().unwrap();
		let file = dir.path().join("msg.txt");
		std::fs::write(&file, "").unwrap();
		assert!(run_editor("false", dir.path(), &file).is_err());
	}

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'expect("reset terminal")' src/popups/externaleditor.rs && grep -qF 'format!("{editor} \"$@\"")' src/popups/externaleditor.rs && grep -qF '!status.success()' src/popups/externaleditor.rs && grep -qE '\bfn run_editor\(' src/popups/externaleditor.rs && grep -qE '\bfn test_run_editor_nonzero_exit_is_error\b' src/popups/externaleditor.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4H: Clipboard Tool Lookup and OSC 52 Flush

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4H: Fail fast when a clipboard tool isn't on PATH (instead of spawning a bare name), ignore an empty WAYLAND_DISPLAY, and flush the OSC 52 escape so it reaches the terminal.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/clipboard.rs
READ_ONLY_REFS:
  - src/clipboard.rs:13-15 (`let binary = which(command).ok().unwrap_or_else(|| PathBuf::from(command));`)
  - src/clipboard.rs:71-83 (`fn copy_string_osc52(text, out)`: `write!(out, "\x1b]52;...\x07", ...)?; Ok(())` with no flush)
  - src/clipboard.rs:118-121 (`if std::env::var("WAYLAND_DISPLAY").is_ok() { return copy_string_wayland(text); }`)
  - src/clipboard.rs tests: clipboard::tests::test_copy_string_osc52
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep the fallback order (wl-copy → OSC 52; xclip → xsel → OSC 52; WSL clip.exe).

[PRE_GATE]
true

[MUTATION_SPEC]
1. exec_copy_with_args: `let binary = which(command).ok().unwrap_or_else(|| PathBuf::from(command));` → `let binary = which(command).map_err(|e| anyhow!("`{command}` not found: {e}"))?;`. Remove `use std::path::PathBuf;` if it becomes unused.
2. copy_string_osc52: after the `write!(...)?;` add `out.flush()?;`.
3. copy_string (unix, non-macos): `std::env::var("WAYLAND_DISPLAY").is_ok()` → `std::env::var("WAYLAND_DISPLAY").is_ok_and(|v| !v.is_empty())`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'unwrap_or_else(|| PathBuf::from(command))' src/clipboard.rs && grep -qF 'out.flush()?;' src/clipboard.rs && grep -qF 'is_ok_and(|v| !v.is_empty())' src/clipboard.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4I: Private Trace Log File

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4I: Create the `-l` trace log with mode 0600 and refuse to write it through a symlink; the trace log records remote URLs and command lines.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/args.rs
READ_ONLY_REFS:
  - src/args.rs:194-212 (`fn setup_logging(path_override: Option<PathBuf>)`: `WriteLogger::init(LevelFilter::Trace, Config::default(), File::create(path)?)?;`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep the default log location (`<cache dir>/gitui/gitui.log`) and the `--logfile` override.

[PRE_GATE]
true

[MUTATION_SPEC]
1. args.rs: add
	fn open_log_file(path: &Path) -> Result<File> {
		if fs::symlink_metadata(path)
			.is_ok_and(|m| m.file_type().is_symlink())
		{
			bail!("refusing to write log through symlink: {}", path.display());
		}
		let mut options = fs::OpenOptions::new();
		options.write(true).create(true).truncate(true);
		#[cfg(unix)]
		{
			use std::os::unix::fs::OpenOptionsExt;
			options.mode(0o600);
		}
		Ok(options.open(path)?)
	}
   and in setup_logging use `open_log_file(&path)?` instead of `File::create(path)?`. Add `bail` to the `anyhow` import and `Path` to the std imports if missing.
2. Tests, in args.rs (add `#[cfg(test)] mod tests` if absent):
	#[cfg(unix)]
	#[test]
	fn test_open_log_file_mode_0600() {
		use std::os::unix::fs::PermissionsExt;
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("gitui.log");
		drop(open_log_file(&path).unwrap());
		assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
	}
	#[cfg(unix)]
	#[test]
	fn test_open_log_file_refuses_symlink() {
		let dir = tempfile::tempdir().unwrap();
		let target = dir.path().join("target.txt");
		fs::write(&target, "keep").unwrap();
		let link = dir.path().join("gitui.log");
		std::os::unix::fs::symlink(&target, &link).unwrap();
		assert!(open_log_file(&link).is_err());
		assert_eq!(fs::read_to_string(&target).unwrap(), "keep");
	}

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qE '\bfn open_log_file\(' src/args.rs && grep -qF 'options.mode(0o600)' src/args.rs && ! grep -qF 'File::create(path)?' src/args.rs && grep -qE '\bfn test_open_log_file_refuses_symlink\b' src/args.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4J: Credential Prompt Clears and Hides Both Inputs

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4J: When the credential prompt is closed, hide and clear both the username and password inputs, so the next prompt doesn't draw a stale password box over the username box (which made the next password get typed visibly into the username field).

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/cred.rs
READ_ONLY_REFS:
  - src/components/cred.rs:147-150 (`fn hide(&mut self) { self.cred = BasicAuthCredential::new(None, None); self.visible = false; }`)
  - src/components/cred.rs:61-68 (draw: draws input_username then input_password when visible)
  - src/components/cred.rs:152-160 (show: shows input_username if no username yet, else input_password)
  - TextInputComponent has `hide()` and `clear()` (used at cred.rs:135-136)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't change how push/pull/fetch react to the prompt closing (that is P2-10, a later phase).

[PRE_GATE]
true

[MUTATION_SPEC]
1. cred.rs hide(): add, after resetting `self.cred`:
		self.input_username.hide();
		self.input_username.clear();
		self.input_password.hide();
		self.input_password.clear();

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'self.input_username.clear();' src/components/cred.rs && test "$(grep -cF 'self.input_password.clear();' src/components/cred.rs)" -ge 2 && test "$(grep -cF 'self.input_username.hide();' src/components/cred.rs)" -ge 2

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 4K: commit.template Expands Only `~`

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 4K: Expand only `~` in commit.template (as git does), not $VARS, so config can't make gitui read a file named by an environment variable into the commit box.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/commit.rs
READ_ONLY_REFS:
  - src/popups/commit.rs:409-415 (`.and_then(|path| { shellexpand::full(path.as_str()).ok().and_then(|path| { PathBuf::from_str(path.as_ref()).ok() }) })`)
  - `shellexpand::tilde(&str) -> Cow<str>` (infallible)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep reading commit.template from repo config (git-parity). Don't change COMMIT_EDITMSG handling.

[PRE_GATE]
true

[MUTATION_SPEC]
1. commit.rs: replace that `.and_then` closure with
					.and_then(|path| {
						PathBuf::from_str(
							shellexpand::tilde(path.as_str()).as_ref(),
						)
						.ok()
					})

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'shellexpand::full' src/popups/commit.rs && grep -qF 'shellexpand::tilde(path.as_str())' src/popups/commit.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

## Phase 5: Crashes and data loss

**Snippets in this phase show intent and shape.** Crates deny `clippy::all, pedantic, nursery`. If clippy rejects a snippet's exact shape (for example `option_if_let_else`), use the form clippy suggests and keep the behavior. The POST_GATE markers are short fixed strings chosen to survive that and rustfmt's `max_width = 70`.

**Order matters:** 5A adds the error boundaries in `App`; later steps (5G and others) rely on them to turn a returned `Err` into an error popup instead of exiting the TUI.

### STEP 5A: Error Boundaries in App

* **Recommended Command:** `/boost`

```text
[STEP_ID]
STEP 5A: Stop a failing internal event or component key handler from exiting gitui: catch the error per event, show it in the error popup, and keep the editor-launch path from leaving input polling switched off.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/app.rs
READ_ONLY_REFS:
  - src/app.rs:722-737 (`fn process_internal_events`: `flags.insert(self.process_internal_event(e)?);` inside `loop { let front = self.queue.pop(); ... }`)
  - src/app.rs:306-317 (`pub fn event`: `if event_pump(&ev, self.components_mut().as_mut_slice())?.is_consumed() {`)
  - src/app.rs:370-397 (InputState::Paused branch: `let changes = self.status_tab.get_files_changes()?; self.commit_popup.show_editor(changes)` then `if let Err(e) = result { ... }` then `self.input.set_polling(true);`)
  - src/gitui.rs:123 (`self.app.event(ev)?;` — any Err returned from App::event ends the program)
  - Audit P1-23: RewordCommit on a dirty repo, Push/PushTags with no remote, ResetHunk/ResetLines failures, stage_remove_all, get_stashes, amend and get_files_changes all reach `?` here today and quit the TUI.
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't change process_internal_event's or process_confirmed_action's bodies in this step.

[PRE_GATE]
true

[MUTATION_SPEC]
1. process_internal_events: replace `flags.insert(self.process_internal_event(e)?);` with an error boundary:
				let is_error_msg =
					matches!(e, InternalEvent::ShowErrorMsg(_));
				let result = self.process_internal_event(e);
				if let Err(err) = &result {
					log::error!("internal event failed: {err:?}");
					if !is_error_msg {
						self.queue.push(InternalEvent::ShowErrorMsg(
							format!("{err}"),
						));
					}
				}
				flags.insert(result.unwrap_or_else(|_| NeedsUpdate::ALL));
   Why: one failing event used to abort the loop and the program. `is_error_msg` stops a failing error popup from re-queueing itself forever.
2. App::event: replace `event_pump(&ev, self.components_mut().as_mut_slice())?` with a call whose error is caught the same way: on Err, `log::error!`, push `InternalEvent::ShowErrorMsg(format!("{err}"))`, and treat the event as consumed (`flags.insert(NeedsUpdate::ALL)`); the later `self.process_queue(flags)?` then shows the popup.
3. InputState::Paused branch: move `get_files_changes()` into the result so its error goes to the existing `if let Err(e) = result` popup and `self.input.set_polling(true);` always runs:
					} else {
						self.status_tab
							.get_files_changes()
							.and_then(|changes| {
								self.commit_popup.show_editor(changes)
							})
					};
   (If the borrow checker rejects the closure, bind `let changes = self.status_tab.get_files_changes();` first and `match`/`and_then` on it.)
4. Tests: none new (App needs a terminal); the full workspace gate is the check.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'flags.insert(self.process_internal_event(e)?)' src/app.rs && grep -qF 'matches!(e, InternalEvent::ShowErrorMsg(_))' src/app.rs && grep -qF 'internal event failed' src/app.rs && ! grep -qF 'self.status_tab.get_files_changes()?;' src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5B: Delete the Right Remote Branch

* **Recommended Command:** `/boost`

```text
[STEP_ID]
STEP 5B: Deleting a remote branch whose name contains `/` (refs/remotes/upstream/feat/login) deletes `feat/login` on `upstream`, instead of `login` on whatever remote the push would pick.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/app.rs
  - src/popups/push.rs
READ_ONLY_REFS:
  - src/app.rs:1060-1086 (`fn delete_remote_branch(&mut self, branch_ref: &str) -> Result<()>`: `branch_ref.rsplit('/').next()` → `InternalEvent::Push(name.to_string(), PushType::Branch, false, true)`, then `self.select_branch_popup.update_branches()?`)
  - src/popups/branchlist.rs:679-690 (the action carries `BranchInfo::reference`, the full ref name, e.g. `refs/remotes/origin/feat/x`)
  - src/popups/push.rs:91-125 (`pub fn push(&mut self, branch, push_type, force, delete) -> Result<()>`), push.rs `fn push_to_remote(...)`: remote = `get_branch_remote(..)` upstream, else `get_default_remote_for_push(..)`
  - asyncgit::sync::get_remotes(repo_path) -> Result<Vec<String>> (asyncgit/src/sync/remotes/mod.rs:86)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep `PushPopup::push`'s signature and the InternalEvent::Push variant unchanged.

[PRE_GATE]
grep -qF 'internal event failed' src/app.rs

[MUTATION_SPEC]
1. app.rs: add a free function
	/// split `refs/remotes/<remote>/<branch>` into (remote, branch),
	/// matching the longest configured remote name (remote and branch
	/// names may both contain `/`)
	fn split_remote_branch_ref<'a>(
		branch_ref: &'a str,
		remotes: &[String],
	) -> Option<(String, &'a str)> {
		let rest = branch_ref.strip_prefix("refs/remotes/")?;
		remotes
			.iter()
			.filter(|r| {
				rest.len() > r.len() + 1
					&& rest.starts_with(r.as_str())
					&& rest.as_bytes()[r.len()] == b'/'
			})
			.max_by_key(|r| r.len())
			.map(|r| (r.clone(), &rest[r.len() + 1..]))
	}
2. app.rs delete_remote_branch: get remotes with `sync::get_remotes(&self.repo.borrow())`; on Err push ShowErrorMsg and return Ok(()). With `Some((remote, branch)) = split_remote_branch_ref(branch_ref, &remotes)` call `self.push_popup.delete_remote_branch(remote, branch.to_string())?`; on None push `InternalEvent::ShowErrorMsg(format!("Failed to find the remote and branch in {branch_ref}"))`. Keep `self.select_branch_popup.update_branches()?` at the end.
3. push.rs: add a field `remote_override: Option<String>` (init `None`). Move the current body of `pub fn push(..)` into a private `fn start_push(&mut self, branch: String, push_type: PushType, force: bool, delete: bool) -> Result<()>`. New:
	pub fn push(&mut self, branch: String, push_type: PushType, force: bool, delete: bool) -> Result<()> {
		self.remote_override = None;
		self.start_push(branch, push_type, force, delete)
	}
	/// delete `branch` on the given `remote` (not the upstream/default remote)
	pub fn delete_remote_branch(&mut self, remote: String, branch: String) -> Result<()> {
		self.remote_override = Some(remote);
		self.start_push(branch, PushType::Branch, false, true)
	}
   In push_to_remote, use `self.remote_override.clone()` when it is Some, before the upstream/default lookup.
4. Tests, in a new `#[cfg(test)] mod tests { use super::split_remote_branch_ref; ... }` at the end of app.rs:
	#[test]
	fn test_split_remote_branch_ref() {
		let remotes = vec![String::from("origin"), String::from("upstream"), String::from("team/fork")];
		assert_eq!(split_remote_branch_ref("refs/remotes/upstream/feat/login", &remotes), Some((String::from("upstream"), "feat/login")));
		assert_eq!(split_remote_branch_ref("refs/remotes/origin/main", &remotes), Some((String::from("origin"), "main")));
		assert_eq!(split_remote_branch_ref("refs/remotes/team/fork/x/y", &remotes), Some((String::from("team/fork"), "x/y")));
		assert_eq!(split_remote_branch_ref("refs/remotes/unknown/main", &remotes), None);
		assert_eq!(split_remote_branch_ref("refs/heads/main", &remotes), None);
		assert_eq!(split_remote_branch_ref("refs/remotes/origin/", &remotes), None);
	}

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF "branch_ref.rsplit('/')" src/app.rs && grep -qE '\bfn split_remote_branch_ref\b' src/app.rs && grep -qE '\bfn test_split_remote_branch_ref\b' src/app.rs && grep -qE '\bpub fn delete_remote_branch\(' src/popups/push.rs && grep -qF 'remote_override' src/popups/push.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5C: Tag Delete Without a Remote, Stage-Remove-All Errors

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5C: After a local tag delete, always refresh the tag list and only ask about deleting the remote tag when a default remote exists; show a stage-remove-all failure as an error popup and don't announce a file move that didn't happen.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/app.rs
  - src/components/changes.rs
READ_ONLY_REFS:
  - src/app.rs:1041-1058 (`fn delete_tag`: `let remote = sync::get_default_remote(&self.repo.borrow())?;` then ConfirmAction(DeleteRemoteTag) then `self.tags_popup.update_tags()?;` — with no remote the `?` skips the tag-list refresh)
  - src/components/changes.rs:266-279 (`} else { self.stage_remove_all()?; } self.queue.push(InternalEvent::StatusLastFileMoved);`; the working-dir branch already uses `try_or_popup!(self, "staging all error:", self.index_add_all())`)
  - `try_or_popup!` macro: src/components/utils/mod.rs:13-26
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qE '\bfn split_remote_branch_ref\b' src/app.rs

[MUTATION_SPEC]
1. app.rs delete_tag, success branch: call `self.tags_popup.update_tags()?;` first, then `if let Ok(remote) = sync::get_default_remote(&self.repo.borrow()) { self.queue.push(InternalEvent::ConfirmAction(Action::DeleteRemoteTag(tag_name, remote))); }`.
2. changes.rs: replace `self.stage_remove_all()?;` with `try_or_popup!(self, "unstaging all error:", self.stage_remove_all());`.
3. changes.rs: push `InternalEvent::StatusLastFileMoved` only when the stage/unstage call succeeded. Restructure so each branch computes `let ok = <call>.is_ok()` (logging/popup on Err as try_or_popup! does) and push StatusLastFileMoved `if ok`. If that's cleaner with a small helper, add one in changes.rs.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'sync::get_default_remote(&self.repo.borrow())?' src/app.rs && grep -qF 'if let Ok(remote) = sync::get_default_remote(' src/app.rs && ! grep -qF 'self.stage_remove_all()?;' src/components/changes.rs && grep -qF '"unstaging all error:"' src/components/changes.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5D: Tree Underflow Panics

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5D: Remove the u8 underflow in filetreelist's show_element when the selected item sits at indent 0, and the usize underflow in StatusTree::selection_updown's search loop.

[SCOPE_LOCK]
ALLOWED_FILES:
  - filetreelist/src/filetreeitems.rs
  - src/components/utils/statustree.rs
READ_ONLY_REFS:
  - filetreelist/src/filetreeitems.rs:214-222 and :254-262 (`} else if item.info().indent() == current_folder.1 - 1 {` and `if item.info().indent() == current_folder.1 - 1 {`; `current_folder.1` is a u8 indent, 0 for top-level items → debug-build panic, wrap in release)
  - src/components/utils/statustree.rs:209-238 (selection_updown: `let mut cur_index_find = current_index;` … loop … `if current_index == 0 { current_index_in_available_selections = 0; break; }` then `cur_index_find -= 1;` — the guard tests the wrong variable, so cur_index_find underflows when the selection isn't in available_selections)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't change which items are shown/selected for valid input.

[PRE_GATE]
grep -qF 'internal event failed' src/app.rs

[MUTATION_SPEC]
1. filetreeitems.rs, both sites: `item.info().indent() == current_folder.1 - 1` → `current_folder.1.checked_sub(1) == Some(item.info().indent())`.
2. statustree.rs selection_updown: `if current_index == 0 {` → `if cur_index_find == 0 {` (the comment "This should never happen" stays).
3. Tests:
   - filetreeitems.rs tests module: `test_show_element_top_level_no_underflow` — build items from paths `["a.txt", "b/c.txt"]` with the module's existing constructor helpers, call the show-element path on the index of "a.txt" (indent 0), and assert it returns without panicking and "a.txt" is visible.
   - statustree.rs tests module: `test_selection_updown_missing_index_no_underflow` — build a StatusTree from items `["a", "b/c"]` with the module's existing helpers, call `selection_updown` with a `current_index` that is not in `available_selections` (collapse "b" first so index of "b/c" is hidden), for both `up` and down, and assert no panic.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'current_folder.1 - 1' filetreelist/src/filetreeitems.rs && test "$(grep -cF 'current_folder.1.checked_sub(1)' filetreelist/src/filetreeitems.rs)" = 2 && grep -qF 'if cur_index_find == 0 {' src/components/utils/statustree.rs && grep -qE '\bfn test_show_element_top_level_no_underflow\b' filetreelist/src/filetreeitems.rs && grep -qE '\bfn test_selection_updown_missing_index_no_underflow\b' src/components/utils/statustree.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5E: Commit List and Status Tree Panics

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5E: Stop the commit list panicking when its selection is past the loaded commits, and the file trees panicking on paths that aren't valid UTF-8.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/commitlist.rs
  - src/components/status_tree.rs
  - src/components/utils/filetree.rs
READ_ONLY_REFS:
  - src/components/commitlist.rs:47 (`commits: IndexSet<CommitId>`), :277 (`entry == &self.commits[self.selection]`), :711 (`let commit = self.commits[self.selection];`) — IndexSet indexing panics when out of range (empty or shrunk list)
  - src/components/status_tree.rs:174-178 (`Path::new(&status_item.path).file_name().and_then(std::ffi::OsStr::to_str).expect("invalid path.");`)
  - src/components/utils/filetree.rs:213 (`String::from(c.to_str().expect("invalid path"))`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qF 'internal event failed' src/app.rs

[MUTATION_SPEC]
1. commitlist.rs:277: `entry == &self.commits[self.selection]` → `self.commits.get_index(self.selection) == Some(entry)`.
2. commitlist.rs:711 selection_highlighted: get the commit with `self.commits.get_index(self.selection)` and return `false` when it is None.
3. status_tree.rs:178: replace `.expect("invalid path.")` with `.unwrap_or(status_item.path.as_str())` (show the full path rather than panic).
4. utils/filetree.rs:213: `String::from(c.to_str().expect("invalid path"))` → `c.to_string_lossy().into_owned()`.
5. Test, in commitlist.rs's tests module: `test_selection_highlighted_empty_list` — build an empty CommitList the way the module's existing tests build one, set highlights to `Some` of any IndexSet, and assert `!list.selection_highlighted()`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'self.commits[self.selection]' src/components/commitlist.rs && ! grep -qF 'expect("invalid path' src/components/status_tree.rs && ! grep -qF 'expect("invalid path' src/components/utils/filetree.rs && grep -qE '\bfn test_selection_highlighted_empty_list\b' src/components/commitlist.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5F: Tag, Blame and Remote List Panics

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5F: Remove the empty-list underflows in the tag list and blame popup, the out-of-bounds blame line lookup, and the off-by-one in the remote list's selection check.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/taglist.rs
  - src/popups/blame_file.rs
  - src/popups/remotelist.rs
READ_ONLY_REFS:
  - src/popups/taglist.rs:391 (`self.tags.as_ref().map_or(0, |tags| tags.len() - 1);`)
  - src/popups/blame_file.rs:329 (`let max_line = blame_result.lines().len() - 1;`), :706 (`.map_or(0, |file_blame| file_blame.lines().len() - 1)`), :788 (`file_blame.lines()[selected]`); :623 `lines()[line_number - 1]` is already guarded by `if line_number == 0` and stays
  - src/popups/remotelist.rs:416-419 (`fn valid_selection`: `self.remote_names.len() >= self.selection as usize`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qF 'internal event failed' src/app.rs

[MUTATION_SPEC]
1. taglist.rs:391: `tags.len() - 1` → `tags.len().saturating_sub(1)`.
2. blame_file.rs:329 and :706: `lines().len() - 1` → `lines().len().saturating_sub(1)`.
3. blame_file.rs:788: `file_blame.lines()[selected].0.as_ref().map(|hunk| hunk.commit_id)` → `file_blame.lines().get(selected).and_then(|line| line.0.as_ref()).map(|hunk| hunk.commit_id)`.
4. remotelist.rs valid_selection: `>=` → `>`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'tags.len() - 1' src/popups/taglist.rs && ! grep -qF 'lines().len() - 1' src/popups/blame_file.rs && ! grep -qF 'file_blame.lines()[selected]' src/popups/blame_file.rs && grep -qF 'self.remote_names.len() > self.selection as usize' src/popups/remotelist.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5G: Branch List Status Error Instead of Panic

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5G: Return the working-dir status error from switch_to_selected_branch (shown as an error popup by STEP 5A's boundary) instead of panicking, and guard its unchecked branch index.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/branchlist.rs
READ_ONLY_REFS:
  - src/popups/branchlist.rs:580-615 (`fn switch_to_selected_branch(&mut self) -> Result<()>`: `sync::status::get_status(..).expect("Could not get status");` then `let selected_branch = &self.branches[self.selection as usize];`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qF 'internal event failed' src/app.rs

[MUTATION_SPEC]
1. branchlist.rs:590: `.expect("Could not get status");` → `?;`.
2. branchlist.rs switch_to_selected_branch: `let selected_branch = &self.branches[self.selection as usize];` → `let Some(selected_branch) = self.branches.get(self.selection as usize) else { return Ok(()); };`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'expect("Could not get status")' src/popups/branchlist.rs && grep -qF 'self.branches.get(self.selection as usize)' src/popups/branchlist.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5H: Cancelling Credentials Cancels Push and Push-Tags

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5H: Pressing Esc on the credential prompt during push or push-tags closes the popup instead of pushing with empty credentials.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/push.rs
  - src/popups/push_tags.rs
READ_ONLY_REFS:
  - src/popups/push.rs:314-333 (`if self.input_cred.get_cred().is_complete() || !self.input_cred.is_visible() { self.push_to_remote(..)?; self.input_cred.hide(); }`)
  - src/popups/push_tags.rs:236-255 (same shape, `self.push_to_remote(Some(..))?`)
  - src/components/cred.rs hide() (STEP 4J): Esc hides the prompt and clears the credentials, so after Esc `is_complete()` is false and `is_visible()` is false
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qE '\bpub fn delete_remote_branch\(' src/popups/push.rs

[MUTATION_SPEC]
1. push.rs and push_tags.rs, in the cred-visible branch: split the condition —
					if self.input_cred.get_cred().is_complete() {
						<existing push_to_remote call>?;
						self.input_cred.hide();
					} else if !self.input_cred.is_visible() {
						// credential prompt cancelled: don't push
						self.hide();
					}

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF '|| !self.input_cred.is_visible()' src/popups/push.rs && ! grep -qF '|| !self.input_cred.is_visible()' src/popups/push_tags.rs && grep -qF "credential prompt cancelled: don't push" src/popups/push.rs && grep -qF "credential prompt cancelled: don't push" src/popups/push_tags.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5I: Cancelling Credentials and Preflight Errors in Pull and Fetch

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5I: Esc on the pull/fetch credential prompt closes the popup instead of fetching with empty credentials, and a preflight error (no remote, no upstream) closes the popup instead of leaving an un-closable modal.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/pull.rs
  - src/popups/fetch.rs
READ_ONLY_REFS:
  - src/popups/pull.rs:70-87 (`pub fn fetch(&mut self, branch: String) -> Result<()>`: `self.show()?;` then `need_username_password_for_fetch(..)?` and the rest), pull.rs:261-273 (cred branch `is_complete() || !self.input_cred.is_visible()` → `self.fetch_from_remote(..)?`)
  - src/popups/fetch.rs:62-80 (`pub fn fetch(&mut self) -> Result<()>`: `self.show()?;` then `need_username_password(..)?`), fetch.rs:174-185 (same cred condition → `self.fetch_all(..)`)
  - Neither popup handles exit_popup itself, so once visible with nothing pending the user can't close it. app.rs:830 and :838 already turn their Err into an error popup.
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qF "credential prompt cancelled: don't push" src/popups/push.rs

[MUTATION_SPEC]
1. pull.rs and fetch.rs cred branch: same split as STEP 5H — `if self.input_cred.get_cred().is_complete() { <existing call>; self.input_cred.hide(); } else if !self.input_cred.is_visible() { // credential prompt cancelled: don't fetch` + `self.hide(); }`.
2. pull.rs `pub fn fetch`: keep `self.branch = branch; self.show()?;`, move the rest into a private `fn start(&mut self) -> Result<()>`, and end with
		let result = self.start();
		if result.is_err() {
			self.hide();
		}
		result
3. fetch.rs `pub fn fetch`: same pattern (private `fn start(&mut self) -> Result<()>` holding everything after `self.show()?;`).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF '|| !self.input_cred.is_visible()' src/popups/pull.rs && ! grep -qF '|| !self.input_cred.is_visible()' src/popups/fetch.rs && grep -qF "credential prompt cancelled: don't fetch" src/popups/pull.rs && grep -qF "credential prompt cancelled: don't fetch" src/popups/fetch.rs && grep -qE '\bfn start\(&mut self\) -> Result<\(\)>' src/popups/pull.rs && grep -qE '\bfn start\(&mut self\) -> Result<\(\)>' src/popups/fetch.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5J: Reset and Checkout Popups Open on the Safe Option

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5J: Every time the reset popup or the checkout-option popup opens, start on the non-destructive choice (Soft reset / keep local changes) instead of whatever was picked last, including Hard or discard-all.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/reset.rs
  - src/popups/checkout_option.rs
READ_ONLY_REFS:
  - src/popups/reset.rs:60 (`kind: ResetType::Soft,` in new), :112-118 (`pub fn open(&mut self, id: CommitId) -> Result<()> { self.show()?; self.commit = Some(id); Ok(()) }`)
  - src/popups/checkout_option.rs:44 (`option: CheckoutOptions::KeepLocalChanges,` in new), :80-86 (`pub fn open(&mut self, branch: BranchInfo) -> Result<()> { self.show()?; self.branch = Some(branch); Ok(()) }`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't add confirm dialogs here (that's P2-08, a later phase).

[PRE_GATE]
true

[MUTATION_SPEC]
1. reset.rs open: add `self.kind = ResetType::Soft;` before `self.show()?;`.
2. checkout_option.rs open: add `self.option = CheckoutOptions::KeepLocalChanges;` before `self.show()?;`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && test "$(grep -cF 'self.kind = ResetType::Soft;' src/popups/reset.rs)" = 1 && test "$(grep -cF 'self.option = CheckoutOptions::KeepLocalChanges;' src/popups/checkout_option.rs)" = 1

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5K: File Discard Prompt Names the File and Warns About Untracked Files

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5K: The "Reset" confirmation for discarding a file or folder names the path and says untracked files are deleted permanently (sync::reset_workdir removes untracked files with force).

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/confirm.rs
  - src/strings.rs
READ_ONLY_REFS:
  - src/strings.rs:202-204 (`pub fn confirm_msg_reset() -> String { "confirm file reset?".to_string() }`; its only caller is confirm.rs:133)
  - src/popups/confirm.rs:131-134 (`Action::Reset(_) => (strings::confirm_title_reset(), strings::confirm_msg_reset(),),`)
  - src/queue.rs:38-41 (`pub struct ResetItem { pub path: String }`)
  - asyncgit/src/sync/reset.rs:30-37 (reset_workdir: checkout with `.remove_untracked(true).force()`)
  - src/strings.rs:1946 (`mod tests`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
true

[MUTATION_SPEC]
1. strings.rs: change the signature to `pub fn confirm_msg_reset(path: &str) -> String` returning
	format!("discard changes to '{path}'?\nuntracked files are deleted permanently and can't be recovered.")
2. confirm.rs: `Action::Reset(_) =>` → `Action::Reset(item) =>` and pass `&item.path` to `strings::confirm_msg_reset`.
3. Test, in strings.rs's tests module:
	#[test]
	fn test_confirm_msg_reset_names_path() {
		let msg = confirm_msg_reset("src/main.rs");
		assert!(msg.contains("'src/main.rs'"));
		assert!(msg.contains("deleted permanently"));
	}

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qE 'pub fn confirm_msg_reset\(path: &str\)' src/strings.rs && ! grep -qF '"confirm file reset?"' src/strings.rs && grep -qF 'Action::Reset(item)' src/popups/confirm.rs && grep -qE '\bfn test_confirm_msg_reset_names_path\b' src/strings.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 5L: Tighten an Existing Trace Log to 0600

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 5L: When the `-l` trace log already exists, set its mode to 0600 too (STEP 4I's `OpenOptions::mode` only applies when the file is created).

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/args.rs
READ_ONLY_REFS:
  - src/args.rs `fn open_log_file(path: &Path) -> Result<File>` (STEP 4I, 08f7022b): symlink check, then `OpenOptions` with `.mode(0o600)` under `#[cfg(unix)]`, then `Ok(options.open(path)?)`
  - src/args.rs tests module: test_open_log_file_mode_0600, test_open_log_file_refuses_symlink
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qE '\bfn open_log_file\(' src/args.rs

[MUTATION_SPEC]
1. open_log_file: open into `let file = options.open(path)?;`, then under `#[cfg(unix)]` call `file.set_permissions(fs::Permissions::from_mode(0o600))?;` (import `std::os::unix::fs::PermissionsExt` in that block), then `Ok(file)`.
2. Test:
	#[cfg(unix)]
	#[test]
	fn test_open_log_file_tightens_existing_mode() {
		use std::os::unix::fs::PermissionsExt;
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("gitui.log");
		fs::write(&path, "old").unwrap();
		fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
		drop(open_log_file(&path).unwrap());
		assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
	}

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'Permissions::from_mode(0o600)' src/args.rs && grep -qE '\bfn test_open_log_file_tightens_existing_mode\b' src/args.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

## Phase 6: Modal flow and stuck states (high-severity usability)

Same conventions as Phase 5: snippets show intent and shape; where clippy (`all, pedantic, nursery`) rejects a shape, use the form clippy suggests and keep the behavior. Markers are short fixed strings. A **modal popup** is one that, while visible, must consume every key event it receives (return `EventState::Consumed`) so nothing leaks to the tabs or popups under it.

### STEP 6A: Popup Event Order Matches Draw Order

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6A: Make the event order of popups the exact reverse of their draw order, so the popup drawn on top always gets keys first, and stop tab/options shortcuts firing underneath an open popup.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/app.rs
READ_ONLY_REFS:
  - src/app.rs:506-546 (`accessors!(self, [ log_search_popup, fuzzy_find_popup, msg_popup, ... help_popup, revlog, status_tab, files_tab, stashing_tab, stashlist_tab ])` — event order: first entry gets events first, see event_pump in src/components/mod.rs:142-153)
  - src/app.rs:548-582 (`setup_popups!(self, [ commit_popup, stashmsg_popup, help_popup, ..., confirm_popup, msg_popup, goto_line_popup ])` — draw order: first is drawn first, i.e. at the bottom)
  - src/app.rs:316-368 (App::event key chain: tab_toggle, tab_toggle_reverse, tab_status/log/files/stashing/stashes via `self.switch_tab(k)?`, cmd_bar_toggle, open_options)
  - `fn any_popup_visible(&self) -> bool` is generated by setup_popups! (src/components/mod.rs:97-104)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't change the setup_popups! (draw) order; don't add or remove popups.

[PRE_GATE]
true

[MUTATION_SPEC]
1. app.rs accessors!: replace the popup part of the list with the exact reverse of the setup_popups! list, followed by the five tabs unchanged:
	goto_line_popup, msg_popup, confirm_popup, options_popup, fetch_popup, pull_popup, push_tags_popup, push_popup, log_search_popup, fuzzy_find_popup, revision_files_popup, rename_branch_popup, create_branch_popup, checkout_option_popup, reset_popup, tags_popup, submodule_popup, update_remote_url_popup, rename_remote_popup, create_remote_popup, remotes_popup, select_branch_popup, tag_commit_popup, external_editor_popup, file_revlog_popup, blame_file_popup, compare_commits_popup, inspect_commit_popup, help_popup, stashmsg_popup, commit_popup,
	revlog, status_tab, files_tab, stashing_tab, stashlist_tab
   Add a comment above it: `// event order: reverse of the draw order in setup_popups! below (topmost popup first)`.
2. App::event: before the key chain, `let popup_open = self.any_popup_visible();` and require `!popup_open` in the conditions for tab_toggle, tab_toggle_reverse, the five tab_* switch keys and open_options (cmd_bar_toggle stays allowed). With a popup open those keys do nothing here.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'event order: reverse of the draw order' src/app.rs && grep -qF 'let popup_open = self.any_popup_visible();' src/app.rs && test "$(grep -cF '!popup_open' src/app.rs)" -ge 4

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 6B: Modal Popups Consume Their Keys

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6B: Stop the revision-files, tag-list and inspect-commit popups from passing keys through to whatever is underneath while they are open.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/revision_files.rs
  - src/popups/taglist.rs
  - src/popups/inspect_commit.rs
READ_ONLY_REFS:
  - src/popups/revision_files.rs:123-145 (event: on exit_popup calls `self.hide_stacked(false)` but doesn't return; then `let res = self.files.event(event)?; ... return Ok(res);` — returns NotConsumed for keys the files component ignores)
  - src/popups/taglist.rs:228-260 (delete_tag / select_tag branches: `self.selected_tag().map_or(Ok(EventState::NotConsumed), ...)` — with no tags the key leaks)
  - src/popups/inspect_commit.rs:174-192 (open_file_tree branch: `return Ok(EventState::NotConsumed);` when `open_request` is None)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep the `hide_stacked(true)` logic in revision_files (it stacks the popup when the files component hid itself).

[PRE_GATE]
grep -qF 'event order: reverse of the draw order' src/app.rs

[MUTATION_SPEC]
1. revision_files.rs event: after `self.hide_stacked(false);` in the exit_popup branch, `return Ok(EventState::Consumed);`. At the end of the visible branch, keep the `hide_stacked(true)` check and return `Ok(EventState::Consumed)` instead of `Ok(res)`.
2. taglist.rs: in the delete_tag and select_tag branches, the `map_or` default becomes `Ok(EventState::Consumed)`.
3. inspect_commit.rs: the `return Ok(EventState::NotConsumed);` in the open_file_tree branch becomes `return Ok(EventState::Consumed);`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'return Ok(res);' src/popups/revision_files.rs && ! grep -qF 'Ok(EventState::NotConsumed),' src/popups/taglist.rs && ! grep -qF 'return Ok(EventState::NotConsumed);' src/popups/inspect_commit.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 6C: Stash List Keys, Fuzzy Finder Navigation, Tab-Switch Refresh

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6C: The stash list consumes the keys it acts on; the fuzzy finder's Up/Down move the selection instead of being re-handled (and reset) by its text input; jumping to the Status tab refreshes it; fuzzy-file selection goes only to the view that opened the finder.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/tabs/stashlist.rs
  - src/popups/fuzzy_find.rs
  - src/app.rs
READ_ONLY_REFS:
  - src/tabs/stashlist.rs:195-221 (after `self.list.event(ev)?`: enter → pop_stash, stash_apply, stash_drop, stash_open → inspect; then always `Ok(EventState::NotConsumed)`)
  - src/popups/fuzzy_find.rs:336-361 (event: exit/enter → `self.hide()`, popup_down/popup_up → `move_selection`, then always `self.find_text.event(event)?` → `update_query()` which resets `self.selection = 0`); fuzzy_find.rs:371-373 (`fn hide(&mut self) { self.visible = false; }`)
  - src/app.rs:825 (`InternalEvent::TabSwitchStatus => self.set_tab(0)?,` — no NeedsUpdate flags, unlike TabSwitch at :826-829)
  - src/app.rs:898-915 (FuzzyFinderChanged with FuzzyFinderTarget::Files updates both `self.files_tab` and `self.revision_files_popup`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't change the accessors!/setup_popups! lists (STEP 6A).

[PRE_GATE]
grep -qF 'event order: reverse of the draw order' src/app.rs

[MUTATION_SPEC]
1. stashlist.rs: each matched key branch (enter, stash_apply, stash_drop, stash_open) returns `Ok(EventState::Consumed)` after its action.
2. fuzzy_find.rs event: when the key matched exit_popup/enter or popup_down/popup_up, `return Ok(EventState::Consumed);` right after handling it, so `find_text` only sees other keys.
3. fuzzy_find.rs hide(): also call `self.find_text.hide();`.
4. app.rs TabSwitchStatus: `=> { self.set_tab(0)?; flags.insert(NeedsUpdate::ALL); }`.
5. app.rs FuzzyFinderChanged, Files target: if `self.revision_files_popup.is_visible()` update only the popup, else only `self.files_tab`. Put the comment `// fuzzy file selection goes only to the view that opened the finder` above it.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && test "$(grep -cF 'return Ok(EventState::Consumed);' src/tabs/stashlist.rs)" -ge 4 && grep -qF 'self.find_text.hide();' src/popups/fuzzy_find.rs && ! grep -qF 'InternalEvent::TabSwitchStatus => self.set_tab(0)?,' src/app.rs && grep -qF 'goes only to the view that opened the finder' src/app.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 6D: Panes Come Back After Blame or File History

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6D: When a tab's file tree hides itself to open blame or file-history (a back-channel the popups use to stack themselves), the owning tab shows it again, so the Files, Status and Stashing panes don't stay blank after the popup closes.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/tabs/files.rs
  - src/components/changes.rs
  - src/tabs/stashing.rs
READ_ONLY_REFS:
  - src/components/revision_files.rs:486-497 (`if self.blame() { self.hide(); ... }`, `if self.file_history() { self.hide(); ... }`) and src/components/status_tree.rs:320, :485 (`self.hide();` before queuing OpenPopup)
  - Popups that rely on this hide to stack themselves and must keep it: src/popups/revision_files.rs:134-139, src/components/commit_details/mod.rs:198-205
  - src/tabs/files.rs:95-101 (FilesTab::event: `return self.files.event(ev);`)
  - src/components/changes.rs:247-250 (ChangesComponent::event: `if self.files.event(ev)?.is_consumed() { return Ok(EventState::Consumed); }`; used by the Status tab for the working-dir and index trees)
  - src/tabs/stashing.rs (the `index: StatusTreeComponent` field; its event handler forwards to `self.index`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't remove the `self.hide()` calls in revision_files.rs/status_tree.rs (the popups need them).

[PRE_GATE]
true

[MUTATION_SPEC]
1. files.rs FilesTab::event: `let res = self.files.event(ev)?; if !self.files.is_visible() { self.files.show()?; } return Ok(res);` (comment: `// the tree hides itself to open blame/history popups; the tab keeps it shown`).
2. changes.rs ChangesComponent::event: after a consumed `self.files.event(ev)?`, `if !self.files.is_visible() { self.files.show()?; }` before returning Consumed (same comment).
3. stashing.rs: in the event handler, after forwarding to `self.index`, `if self.is_visible() && !self.index.is_visible() { self.index.show()?; }` (same comment).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'the tab keeps it shown' src/tabs/files.rs && grep -qF 'the tab keeps it shown' src/components/changes.rs && grep -qF 'the tab keeps it shown' src/tabs/stashing.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 6E: Search and Highlighting Finish While a Tab Is Hidden

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6E: Process the log search's CommitFilter notification and the Files tab's async (syntax-highlighting) notifications even while their tab is hidden, so switching tabs mid-search or mid-highlight doesn't leave them stuck at "Searching" / "(0%)".

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/tabs/revlog.rs
  - src/tabs/files.rs
READ_ONLY_REFS:
  - src/tabs/revlog.rs:153-170 (`pub fn update_git(&mut self, ev)`: `if self.visible { match ev { ... AsyncGitNotification::CommitFilter => { self.update_search_state(); } ... } }`)
  - src/tabs/files.rs:51-59 (`pub fn update_async(&mut self, ev: AsyncNotification)`: `if self.is_visible() { self.files.update(ev)?; }`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Other notification arms in revlog keep their visibility gate.

[PRE_GATE]
grep -qF 'the tab keeps it shown' src/tabs/files.rs

[MUTATION_SPEC]
1. revlog.rs update_git: handle `AsyncGitNotification::CommitFilter` before the `if self.visible` gate (call `self.update_search_state();` and return Ok(())), and remove its arm from the gated match. Comment above it: `// search results must land even while the tab is hidden`.
2. files.rs update_async: drop the `if self.is_visible()` gate and always call `self.files.update(ev)?;` (comment: `// async results must land even while the tab is hidden`).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'async results must land even while the tab is hidden' src/tabs/files.rs && grep -qF 'search results must land even while the tab is hidden' src/tabs/revlog.rs && test "$(grep -cF 'AsyncGitNotification::CommitFilter' src/tabs/revlog.rs)" = 1

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 6F: File Tree Keeps a Valid Selection

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6F: FileTree::select_file keeps the current selection when the path isn't found (instead of clearing it and breaking keyboard navigation), re-reveals a re-selected file, recalculates the visual selection after collapse/expand, and collapsing a folder recursively no longer collapses the next sibling folder.

[SCOPE_LOCK]
ALLOWED_FILES:
  - filetreelist/src/filetree.rs
  - filetreelist/src/filetreeitems.rs
READ_ONLY_REFS:
  - filetreelist/src/filetree.rs:180-197 (`pub fn select_file(&mut self, path: &Path) -> bool`: `let new_selection = ...position(..)`; `if new_selection == self.selection { return false; }`; `self.selection = new_selection;` (None on a miss); show_element; recompute visual_selection)
  - filetreelist/src/filetree.rs:69-75 (collapse_but_root), :106-117 (collapse_recursive, expand_recursive) — none recompute `self.visual_selection`
  - filetreelist/src/filetreeitems.rs:129-152 (collapse: in the loop, `if recursive && item.kind().is_path() { item.collapse_path(); }` runs before `if item_path.starts_with(&path) { item.hide(); } else { break; }`, so the first non-descendant folder gets collapsed too)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep select_file's `bool` return meaning "selection changed".

[PRE_GATE]
true

[MUTATION_SPEC]
1. filetree.rs select_file: on a miss (`position` is None) return false and leave `self.selection` unchanged. On a hit, always call `self.items.show_element(idx)` and recompute `self.visual_selection`, then return whether the selection changed.
2. filetree.rs: in collapse_but_root, collapse_recursive and expand_recursive, add `self.visual_selection = self.calc_visual_selection();` after the change.
3. filetreeitems.rs collapse: move the `if recursive && item.kind().is_path() { item.collapse_path(); }` inside the `if item_path.starts_with(&path)` branch (before `item.hide()`).
4. Tests:
   - filetree.rs tests: `test_select_file_miss_keeps_selection` — tree from `["a/b.txt", "c.txt"]`, `select_file(Path::new("c.txt"))` → true; `select_file(Path::new("missing.txt"))` → false; `selected_file()` still `c.txt`.
   - filetreeitems.rs tests: `test_collapse_recursive_keeps_sibling` — items from `["a/x/1.txt", "b/2.txt"]`, collapse the index of `a` with `recursive = true`, assert folder `b` is still expanded (not collapsed) and `b/2.txt` is visible.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qE '\bfn test_select_file_miss_keeps_selection\b' filetreelist/src/filetree.rs && grep -qE '\bfn test_collapse_recursive_keeps_sibling\b' filetreelist/src/filetreeitems.rs && test "$(grep -cF 'self.visual_selection = self.calc_visual_selection();' filetreelist/src/filetree.rs)" -ge 4

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 6G: Stashing Tab Honors the Untracked Toggle

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6G: The Stashing tab's file list hides untracked files when "stash untracked" is off, and the toggle survives switching tabs instead of resetting from git config on every show.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/tabs/stashing.rs
READ_ONLY_REFS:
  - src/tabs/stashing.rs:71-80 (`pub fn update(&self)`: `//TODO: support options` then `.fetch(&StatusParams::new(StatusType::Both, None))?;`)
  - src/tabs/stashing.rs:258-273 (show(): re-derives `self.options.stash_untracked = !config_untracked_files.include_none();` on every show)
  - asyncgit StatusParams::new(status_type: StatusType, config: Option<ShowUntrackedFilesConfig>) (asyncgit/src/status.rs:32-40); ShowUntrackedFilesConfig::{No, Normal, All} (asyncgit/src/sync/config.rs:14-22). `None` means "use the repo's status.showUntrackedFiles".
  - keep_index doesn't change what gets stashed (git stashes everything and keeps the index), so the list stays the same for it.
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qF 'the tab keeps it shown' src/tabs/stashing.rs

[MUTATION_SPEC]
1. update(): `let untracked = (!self.options.stash_untracked).then_some(ShowUntrackedFilesConfig::No);` and fetch with `StatusParams::new(StatusType::Both, untracked)`. Remove the `//TODO: support options` comment. Import ShowUntrackedFilesConfig from asyncgit::sync.
2. Add a field `options_initialized: bool` (false in new). In show(), derive `stash_untracked` from config only when `!self.options_initialized`, then set it to true.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'TODO: support options' src/tabs/stashing.rs && grep -qF 'then_some(ShowUntrackedFilesConfig::No)' src/tabs/stashing.rs && grep -qF 'options_initialized' src/tabs/stashing.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 6H: REMOTES Flag, Branch List Refresh and Truncation

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6H: Give NeedsUpdate::REMOTES its own bit (it was ALL|BRANCHES), refresh the branch list after a fetch, drop every `*/HEAD` remote branch, and actually truncate long commit messages and submodule paths before appending "...".

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/queue.rs
  - src/popups/branchlist.rs
  - src/popups/submodules.rs
READ_ONLY_REFS:
  - src/queue.rs:20-34 (`const REMOTES = 0b1001;` next to ALL = 0b001 and BRANCHES = 0b1000)
  - src/popups/branchlist.rs:320-326 (`.position(|b| b.name.ends_with("/HEAD")).map(|idx| self.branches.remove(idx));` — removes only the first)
  - src/popups/branchlist.rs:332-341 (update_git: `if self.is_visible() && ev == AsyncGitNotification::Push {`)
  - src/popups/branchlist.rs:505-509 (`commit_message.unicode_truncate(commit_message_length.saturating_sub(THREE_DOTS_LENGTH));` — return value (truncated &str, width) is discarded; compare the branch_name code just below, which uses `.0`)
  - src/popups/submodules.rs:363-367 (same discarded `module_path.unicode_truncate(..)`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
true

[MUTATION_SPEC]
1. queue.rs: `const REMOTES = 0b1001;` → `const REMOTES = 0b1_0000;`.
2. branchlist.rs update_branches: replace the position/remove with `self.branches.retain(|b| !b.name.ends_with("/HEAD"));` (still only when `!self.local`).
3. branchlist.rs update_git: refresh on `AsyncGitNotification::Push | AsyncGitNotification::Fetch` (use `matches!`).
4. branchlist.rs commit message: `commit_message = commit_message.unicode_truncate(..).0.to_string();` then append THREE_DOTS.
5. submodules.rs: `module_path = module_path.unicode_truncate(..).0.to_string();` then append THREE_DOTS.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'const REMOTES = 0b1_0000;' src/queue.rs && grep -qF 'retain(|b| !b.name.ends_with("/HEAD"))' src/popups/branchlist.rs && grep -qF 'AsyncGitNotification::Fetch' src/popups/branchlist.rs && grep -qF 'commit_message = commit_message' src/popups/branchlist.rs && grep -qF 'module_path = module_path' src/popups/submodules.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 6I: Show Fetch Errors

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6I: Surface a failed fetch (bad credentials, unreachable remote) as an error popup instead of closing the fetch popup silently.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/fetch_job.rs
  - src/popups/fetch.rs
READ_ONLY_REFS:
  - asyncgit/src/fetch_job.rs:13-68 (`enum JobState { Request(Option<BasicAuthCredential>), Response(Result<()>) }`, `pub struct AsyncFetchJob { state: Arc<Mutex<Option<JobState>>>, repo }`; run() stores `JobState::Response(result)` and nothing reads it)
  - src/popups/fetch.rs:107-118 (update_git → update(): `self.pending = self.async_fetch.is_pending(); ... if !self.pending { self.hide(); ...`)
  - AsyncSingleJob::take_last(&self) -> Option<J> (asyncgit/src/asyncjob/mod.rs:107)
  - src/popups/fetch.rs has `queue: Queue`; InternalEvent::ShowErrorMsg(String)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep AsyncFetchJob::new's signature.

[PRE_GATE]
true

[MUTATION_SPEC]
1. fetch_job.rs: add
	/// the fetch's error message, once the job has finished with an error
	pub fn error(&self) -> Option<String> {
		let state = self.state.lock().ok()?;
		match state.as_ref()? {
			JobState::Response(Err(e)) => Some(e.to_string()),
			_ => None,
		}
	}
2. fetch.rs update(): when `!self.pending`, before hiding: `if let Some(err) = self.async_fetch.take_last().and_then(|job| job.error()) { self.queue.push(InternalEvent::ShowErrorMsg(format!("fetch failed:\n{err}"))); }`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qE 'pub fn error\(&self\) -> Option<String>' asyncgit/src/fetch_job.rs && grep -qF 'fetch failed:' src/popups/fetch.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 6J: Blame Errors Stop the Spinner

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 6J: When blaming a file fails, show the error and close the blame popup, instead of logging it and re-requesting forever behind a spinner.

[SCOPE_LOCK]
ALLOWED_FILES:
  - asyncgit/src/blame.rs
  - src/popups/blame_file.rs
READ_ONLY_REFS:
  - asyncgit/src/blame.rs:33-40 (`pub struct AsyncBlame { current, last, sender, pending, repo }`), :107-125 (spawned task: `Err(err) => { log::error!("get_blame_helper error: {err}"); true }` — notifies but leaves `last` unchanged)
  - asyncgit/src/blame.rs:58-64 (`pub fn last(&self) -> Result<Option<(BlameParams, FileBlame)>>`)
  - src/popups/blame_file.rs:443-482 (update(): in BlameProcess::GettingBlame, if `last()` doesn't match `params` it calls `async_blame.request(params.clone())?` again → endless loop on a persistent error)
  - src/popups/blame_file.rs:89 (`queue: Queue`), :379 (`fn hide_stacked(&mut self, stack: bool)`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep AsyncBlame's existing pub fns.

[PRE_GATE]
true

[MUTATION_SPEC]
1. blame.rs: add a field `last_error: Arc<Mutex<Option<String>>>` (init `Arc::new(Mutex::new(None))`); clone it into the spawned task and, in the `Err(err)` arm, store `err.to_string()` in it (keep the log line). Add
	/// takes the error of the last failed blame, if any
	pub fn take_last_error(&self) -> Result<Option<String>> {
		Ok(self.last_error.lock()?.take())
	}
2. blame_file.rs update(): inside the GettingBlame branch, first check `if let Some(err) = async_blame.take_last_error()? { self.queue.push(InternalEvent::ShowErrorMsg(format!("blame failed:\n{err}"))); self.hide_stacked(false); return Ok(()); }` (restructure to satisfy the borrow checker: read the error before calling self methods).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qE 'pub fn take_last_error\(&self\)' asyncgit/src/blame.rs && grep -qF 'last_error' asyncgit/src/blame.rs && grep -qF 'blame failed:' src/popups/blame_file.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

## Phase 7: Stuck and wrong states

Same conventions as Phases 5–6: snippets show intent and shape; follow clippy's preferred form where it rejects one; markers are short fixed strings.

### STEP 7A: Bad Tab Index, Typechange Symbol, Key Symbol Errors

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7A: A persisted tab index out of range falls back to the Status tab instead of failing every draw; type-changed files show `T`; a broken key_symbols.ron is logged instead of silently ignored.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/options.rs
  - src/components/status_tree.rs
  - src/keys/symbols.rs
READ_ONLY_REFS:
  - src/options.rs:61-63 (`pub const fn current_tab(&self) -> usize { self.data.tab }`); src/app.rs:290-297 draws tabs 0..=4 and hits `_ => bail!("unknown tab")` for anything else
  - src/components/status_tree.rs:141-150 (item_status_char: `StatusItemType::Typechange => ' ',`)
  - src/keys/symbols.rs:55-63 (`KeySymbolsFile::read_file(file).unwrap_or_default();`); compare src/keys/key_list.rs:235-246, which logs `KeysList parse error`
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Keep the options file format.

[PRE_GATE]
true

[MUTATION_SPEC]
1. options.rs current_tab: return `self.data.tab` when it is `<= 4` (the five tabs), else `0`. Add a `// tabs 0..=4 exist; an older or hand-edited options file may hold more` comment.
2. status_tree.rs: `StatusItemType::Typechange => ' ',` → `StatusItemType::Typechange => 'T',`.
3. symbols.rs init: replace `.unwrap_or_default()` with `.unwrap_or_else(|e| { log::error!("KeySymbols parse error: {e}"); KeySymbolsFile::default() })`.
4. Test, in options.rs (add `#[cfg(test)] mod tests` if absent, using the existing `test_env()` constructor at options.rs:35-42): set `data.tab = 7` and assert `current_tab() == 0`; set `data.tab = 3` and assert `current_tab() == 3`. Name it `test_current_tab_out_of_range`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF "StatusItemType::Typechange => 'T'," src/components/status_tree.rs && grep -qF 'KeySymbols parse error' src/keys/symbols.rs && ! grep -qF 'read_file(file).unwrap_or_default()' src/keys/symbols.rs && grep -qE '\bfn test_current_tab_out_of_range\b' src/options.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7B: Diff Selection, Emptiness and Empty Line Discards

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7B: Shift+Down in the diff no longer selects one line past the end; the diff exposes whether it has content; discarding lines with nothing selected doesn't open a "discard 0 lines?" prompt.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/diff.rs
READ_ONLY_REFS:
  - src/components/diff.rs:75-88 (`Selection::modify(&mut self, direction, max: usize)`: Down → `cmp::min(old_end + 1, max)`), :274-278 (`fn modify_selection`: `self.selection.modify(direction, self.lines_count());` — passes the line *count*, so the end can reach an index one past the last line), :224-227 (move_selection uses `diff.lines.saturating_sub(1)` as max)
  - src/components/diff.rs:265-267 (`fn lines_count(&self) -> usize`)
  - src/components/diff.rs:562-569 (`fn reset_lines(&self)`: always queues `Action::ResetLines(path, self.selected_lines())`), :593 (`fn selected_lines(&self) -> Vec<DiffLinePosition>`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
true

[MUTATION_SPEC]
1. modify_selection: pass `self.lines_count().saturating_sub(1)` as the max.
2. Add `pub fn is_empty(&self) -> bool { self.lines_count() == 0 }` to DiffComponent (make it `const` only if clippy asks).
3. reset_lines: `let lines = self.selected_lines(); if lines.is_empty() { return; }` then queue the action with `lines`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'self.lines_count().saturating_sub(1)' src/components/diff.rs && grep -qE 'pub (const )?fn is_empty\(&self\) -> bool' src/components/diff.rs && grep -qF 'if lines.is_empty() {' src/components/diff.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7C: Status Tab Focus, Fetch and Push Availability

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7C: When the diff the Status tab focused goes empty (last hunk staged), focus returns to the file list; fetch is offered whenever a fetch remote exists (no upstream needed); push isn't offered on a detached HEAD; remotes are re-checked on every update.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/tabs/status.rs
READ_ONLY_REFS:
  - src/tabs/status.rs:458-482 (`fn update_status`: re-sets items, `self.update_diff()?;`, then the git_action_executed focus fallback for WorkDir/Stage only)
  - src/tabs/status.rs:69 (`diff_target: DiffTarget`), :332-352 (`fn switch_focus(&mut self, f: Focus) -> Result<bool>`)
  - src/tabs/status.rs:384-400 (`pub fn update(&mut self)`), :417-428 (`fn check_remotes(&mut self)`, today only run on AsyncGitNotification::Branches and in show())
  - src/tabs/status.rs:612-624 (`fn can_push`: `is_none_or(|state| state.ahead > 0) && has_remote_for_push` — a detached HEAD has no branch state, so push is offered; `const fn can_fetch`: `has_remote_for_fetch && self.git_branch_state.is_some()` — fetch is blocked without an upstream)
  - src/tabs/status.rs:884-896 (fetch and pull keys both gated by can_fetch)
  - DiffComponent::is_empty (STEP 7B); `self.git_branch_name.last()` is the HEAD shorthand, which is `"HEAD"` when detached (asyncgit/src/sync/branch/mod.rs:31-45)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
grep -qE 'pub (const )?fn is_empty\(&self\) -> bool' src/components/diff.rs

[MUTATION_SPEC]
1. update_status: at the end, `if self.focus == Focus::Diff && self.diff.is_empty() { self.switch_focus(if self.diff_target == DiffTarget::Stage { Focus::Stage } else { Focus::WorkDir })?; }`.
2. update(): call `self.check_remotes();` (inside the visible branch).
3. can_fetch → `self.remotes.has_remote_for_fetch` only. Add `fn can_pull(&self) -> bool` with the old condition (`has_remote_for_fetch && git_branch_state.is_some()`) and use it for the pull key and pull command hint; the fetch key and hint use can_fetch.
4. can_push: also require `self.git_branch_name.last().is_some_and(|name| name != "HEAD")` (comment `// detached HEAD: nothing to push to`).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'self.diff.is_empty()' src/tabs/status.rs && grep -qE '\bfn can_pull\(&self\)' src/tabs/status.rs && grep -qF 'detached HEAD: nothing to push to' src/tabs/status.rs && ! grep -A3 -F 'fn can_fetch' src/tabs/status.rs | grep -qF 'git_branch_state'

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7D: Revlog Esc, Command Hints, Force-Push Branch Name

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7D: Esc in the log closes the commit details pane when no search is active; the Quit hint shows the real quit key; the diff Home/End hint lists only Home/End; the force-push confirm shows the full branch name.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/tabs/revlog.rs
  - src/strings.rs
  - src/popups/confirm.rs
READ_ONLY_REFS:
  - src/tabs/revlog.rs:467-477 (exit_popup branch: cancels/clears search, always `return Ok(EventState::Consumed)`; never closes details), :464 (`self.commit_details.toggle_visible()?;` on enter)
  - src/strings.rs:1324-1333 (`pub fn quit`: `"Quit [{}]"` with `key_config.keys.exit` only; the quit key is `key_config.keys.quit`, used by App::check_quit)
  - src/strings.rs:754-768 (`pub fn diff_home_end`: "Jump up/down [{},{},{},{}]" with home, end, move_up, move_down; diff.rs only jumps on home/end)
  - src/popups/confirm.rs:188-196 (ForcePush: `branch.rsplit('/').next().expect(...)` — `feat/login` shows as `login`; the action carries the branch shorthand from src/tabs/status.rs:567-570)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
true

[MUTATION_SPEC]
1. revlog.rs exit_popup branch: add a final `else if self.commit_details.is_visible() { self.commit_details.toggle_visible()?; self.update()?; }` after the two search branches. Comment above it: `// Esc closes the details pane when no search is active`.
2. strings.rs quit: `"Quit [{}/{}]"` with `key_config.get_hint(key_config.keys.quit)` and `key_config.get_hint(key_config.keys.exit)`.
3. strings.rs diff_home_end: `"Jump up/down [{},{}]"` with home and end only.
4. confirm.rs ForcePush: pass `branch` itself (strip a leading `refs/heads/` if present: `branch.strip_prefix("refs/heads/").unwrap_or(branch)`), removing the rsplit/expect.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'Esc closes the details pane when no search is active' src/tabs/revlog.rs && grep -qF '"Quit [{}/{}]"' src/strings.rs && grep -qF '"Jump up/down [{},{}]"' src/strings.rs && ! grep -qF 'There was no / in the head reference' src/popups/confirm.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7E: Single-Line Inputs Ignore Multi-Line Keys; Vim Keymap Collision

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7E: Single-line and password text inputs no longer insert a Tab or swallow Ctrl+n/Ctrl+p/Up/Down (so popups bound to popup_down/popup_up still get them), the character counter counts characters not bytes, and the vim example keymap no longer binds status_reset_item to the undo-commit key.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/textinput.rs
  - vim_style_key_config.ron
READ_ONLY_REFS:
  - src/components/textinput.rs:27 (`pub enum InputType`), :51 (`input_type: InputType`), :263 (`fn process_inputs(ta: &mut TextArea<'_>, input: &Input) -> bool` — handles `Key::Tab` → `ta.insert_tab()`, Ctrl+n / Down → CursorMove::Down, Ctrl+p / Up → CursorMove::Up, for every input type)
  - src/components/textinput.rs:675-711 (event: exit_popup, newline (Multiline only), else `Self::process_inputs(ta, &input)`; returns Consumed only when `modified`)
  - src/components/textinput.rs:205-206 (`let count = self.get_text().len();` in draw_char_count)
  - vim_style_key_config.ron:29 (`status_reset_item: Some(( code: Char('U'), modifiers: "SHIFT")),`); src/keys/key_list.rs:207 (default `undo_commit` is Shift+U); the default status_reset_item is Shift+D (key_list.rs:172), which doesn't collide in the Status tab
  - src/keys/key_list.rs:257-264 (test_apply_vim_style_example parses the ron file)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Multiline inputs keep every current key.

[PRE_GATE]
true

[MUTATION_SPEC]
1. textinput.rs: add
	/// keys that only make sense in a multi-line editor
	fn is_multiline_only(input: &Input) -> bool {
		matches!(
			input,
			Input { key: Key::Tab, ctrl: false, alt: false, .. }
				| Input { key: Key::Char('n' | 'p'), ctrl: true, alt: false, .. }
				| Input { key: Key::Up | Key::Down, ctrl: false, alt: false, .. }
		)
	}
   and in event, before `Self::process_inputs(ta, &input)`: `} else if self.input_type != InputType::Multiline && Self::is_multiline_only(&input) { false } else { Self::process_inputs(ta, &input) }` so these keys return NotConsumed for single-line and password inputs.
2. textinput.rs draw_char_count: `self.get_text().len()` → `self.get_text().chars().count()`.
3. vim_style_key_config.ron: delete the `status_reset_item: Some(( code: Char('U'), modifiers: "SHIFT")),` line (falls back to the default Shift+D).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qE '\bfn is_multiline_only\(' src/components/textinput.rs && grep -qF 'get_text().chars().count()' src/components/textinput.rs && ! grep -qF 'status_reset_item' vim_style_key_config.ron

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7F: Revision Files Don't Stay at "Loading ..."

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7F: If listing a commit's files fails, show the error and an empty list instead of "Loading ..." forever.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/revision_files.rs
READ_ONLY_REFS:
  - src/components/revision_files.rs:125-150 (`fn refresh_files`: `if let Ok(last) = result.result { ... self.files = Some(last); }` — on Err nothing is set, so `self.files` stays None)
  - src/components/revision_files.rs:330-345 (draw: `if self.files.is_some() { list } else { loading_text }`)
  - the component holds `queue: Queue` (revision_files.rs:47); `FileTree` derives Default (filetreelist/src/filetree.rs:33)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
true

[MUTATION_SPEC]
1. refresh_files: turn the `if let Ok(last) = result.result` into a match; on `Err(e)`: `log::error!("tree files error: {e}")`, push `InternalEvent::ShowErrorMsg(format!("listing files failed:\n{e}"))`, set `self.tree = FileTree::default()` and `self.files = Some(Vec::new())`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'tree files error' src/components/revision_files.rs && grep -qF 'self.files = Some(Vec::new())' src/components/revision_files.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7G: Status Tree Keeps Its Selection; Commit List Clamps

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7G: After a status refresh the file tree re-finds the previously selected file by exact path (the binary search assumed sorted paths, which tree order isn't, so the selection jumped); the commit list clamps its selection when the list shrinks.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/utils/statustree.rs
  - src/components/commitlist.rs
READ_ONLY_REFS:
  - src/components/utils/statustree.rs:192-208 (`fn find_last_selection`: `self.tree.items().binary_search_by(|e| e.info.full_path.as_str().cmp(last_selection))`; tree order interleaves folders and files, e.g. `a-b` sorts before `a/`, so the search misses)
  - src/components/commitlist.rs:208-215 (`pub fn set_commits(&mut self, commits: IndexSet<CommitId>)`: replaces commits without touching `self.selection`)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
true

[MUTATION_SPEC]
1. statustree.rs find_last_selection: first look for an exact match with `self.tree.items().iter().position(|e| e.info.full_path == last_selection)`; if none, fall back to the index of the first item whose path sorts after `last_selection`, else the last item. Remove the `binary_search_by`.
2. commitlist.rs set_commits: after replacing commits, `self.selection = self.selection.min(self.commits.len().saturating_sub(1));`.
3. Test, in statustree.rs tests: `test_keep_selection_with_dash_and_slash` — items `["a-b", "a/c"]` (tree order puts the `a/` folder and `a/c` relative to `a-b` differently than string order), select `a/c`, call `update` with the same items again, assert the selected item's path is still `a/c`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && ! grep -qF 'binary_search_by' src/components/utils/statustree.rs && grep -qF 'self.selection.min(self.commits.len().saturating_sub(1))' src/components/commitlist.rs && grep -qE '\bfn test_keep_selection_with_dash_and_slash\b' src/components/utils/statustree.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7H: Remote Tag and Remote URL Errors Are Shown

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7H: The tag list reports a failed remote-tags lookup, and "update remote URL" tells the user when the URL can't be read instead of doing nothing.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/taglist.rs
  - src/popups/remotelist.rs
READ_ONLY_REFS:
  - src/popups/taglist.rs:344-353 (`if let Some(Ok(missing_remote_tags)) = job.result() { ... }` — an Err is dropped)
  - src/popups/remotelist.rs:461-472 (`fn update_remote_url(&self)`: `if let Ok(Some(url)) = remote_url { push UpdateRemoteUrl }` — Ok(None) and Err do nothing)
  - both popups hold `queue: Queue`; InternalEvent::ShowErrorMsg(String)
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES.

[PRE_GATE]
true

[MUTATION_SPEC]
1. taglist.rs: match `job.result()`: `Some(Ok(tags))` as now; `Some(Err(e))` → `self.queue.push(InternalEvent::ShowErrorMsg(format!("checking remote tags failed:\n{e}")))`.
2. remotelist.rs update_remote_url: `Ok(None)` → ShowErrorMsg(format!("remote '{remote_name}' has no URL")); `Err(e)` → ShowErrorMsg(format!("reading remote URL failed:\n{e}")).

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qF 'checking remote tags failed' src/popups/taglist.rs && grep -qF 'has no URL' src/popups/remotelist.rs && grep -qF 'reading remote URL failed' src/popups/remotelist.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7I: Fuzzy Finder: Enter Picks, Esc Cancels

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7I: The fuzzy finder applies a match only on Enter; Esc closes it without changing the branch list or file tree selection.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/popups/fuzzy_find.rs
READ_ONLY_REFS:
  - src/popups/fuzzy_find.rs:108-127 (`fn refresh_selection`: pushes `InternalEvent::FuzzyFinderChanged(idx, self.contents[idx].clone(), target)` whenever the highlighted match changes — live preview, so Esc can't undo it)
  - src/popups/fuzzy_find.rs event (after STEP 6C): exit_popup or enter → `self.hide(); return Ok(EventState::Consumed);`
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Don't change FuzzyFinderChanged's shape or the app.rs handler.

[PRE_GATE]
grep -qF 'self.find_text.hide();' src/popups/fuzzy_find.rs

[MUTATION_SPEC]
1. refresh_selection: keep updating `self.selected_index`, but stop pushing FuzzyFinderChanged there.
2. Add `fn commit_selection(&self)` that pushes FuzzyFinderChanged for `self.selected_index` (when both it and `self.target` are Some) — the code moved out of refresh_selection.
3. event: split the exit/enter branch — enter → `self.commit_selection(); self.hide();`; exit_popup → `self.hide();` only. Both return Consumed.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && grep -qE '\bfn commit_selection\(&self\)' src/popups/fuzzy_find.rs && test "$(grep -cF 'InternalEvent::FuzzyFinderChanged(' src/popups/fuzzy_find.rs)" = 1 && grep -qF 'self.commit_selection();' src/popups/fuzzy_find.rs

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7J: Status Tab and Diff Behave Correctly

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7J: the diff selection, focus and fetch/pull/push availability are correct in the Status tab.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/*
  - src/tabs/status.rs
  - src/strings.rs
READ_ONLY_REFS:
  - neb: BUG-0052, BUG-0062 (part), BUG-0056 (part), TASK-0082. Replaces 7B + 7C
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Pull availability keeps its current rule.

[PRE_GATE]
true

[MUTATION_SPEC]
Outcome step: each item is a required behavior. The implementation is the coder's choice (repo-rules, Outcome steps).
1. Extending the diff selection down stops at the last line.
   Proof: test `t7j_selection_stops_at_last_line`.
2. Discarding lines with no added or deleted line selected opens no prompt.
   Proof: test `t7j_empty_discard_no_prompt`.
3. If the focused diff has finished loading and has no content, focus returns to the file list it came from. A diff that is still loading keeps focus.
   Proof: test `t7j_focus_leaves_only_loaded_empty_diff`.
4. Fetch needs only a fetch remote, not an upstream. Pull still needs both.
   Proof: test `t7j_fetch_without_upstream`.
5. Push is not offered on a detached HEAD.
   Proof: test `t7j_no_push_when_detached`.
6. Remote availability is refreshed whenever the visible Status tab updates.
   Proof: validator reads the code path.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && test "$(cargo test --workspace --locked t7j_ 2>&1 | grep -cE 't7j_.* ok$')" -ge 5

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7K: Keys and Hints Do What They Say

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7K: keys reach the right handler and their hints are accurate.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/textinput.rs
  - src/popups/*
  - src/tabs/revlog.rs
  - src/strings.rs
  - vim_style_key_config.ron
READ_ONLY_REFS:
  - neb: BUG-0053, BUG-0063, BUG-0056 (part). Replaces 7E + 7I + the key and hint parts of 7D
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Multiline inputs keep every current key. The FuzzyFinderChanged event keeps its shape.

[PRE_GATE]
true

[MUTATION_SPEC]
Outcome step: each item is a required behavior. The implementation is the coder's choice (repo-rules, Outcome steps).
1. Single-line and password inputs don't consume Tab, Up, Down, Ctrl+n or Ctrl+p.
   Proof: test `t7k_single_line_passes_keys_through`.
2. The input character counter counts characters, not bytes.
   Proof: test `t7k_counter_counts_chars`.
3. The vim example keymap has no status_reset_item / undo_commit collision.
   Proof: test `t7k_vim_keymap_no_collision`.
4. The fuzzy finder applies a match only on Enter. Esc and moving the highlight apply nothing.
   Proof: test `t7k_fuzzy_applies_only_on_enter`.
5. The Quit hint includes the real quit key as well as the exit key.
   Proof: test `t7k_quit_hint_shows_quit_key`.
6. The diff jump hint lists only Home and End.
   Proof: test `t7k_jump_hint_home_end_only`.
7. In the log, Esc closes commit details when no search is active. An active search is still cancelled first.
   Proof: validator reads the code path.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && test "$(cargo test --workspace --locked t7k_ 2>&1 | grep -cE 't7k_.* ok$')" -ge 6

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7L: Failures and Confirmations Tell the Truth

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7L: failed lookups are shown and the force-push prompt names the real branch.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/revision_files.rs
  - src/popups/*
  - src/strings.rs
READ_ONLY_REFS:
  - neb: BUG-0055, BUG-0062 (rest), BUG-0056 (part). Replaces 7F + 7H + the force-push part of 7D
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. Successful lookups behave as before.

[PRE_GATE]
true

[MUTATION_SPEC]
Outcome step: each item is a required behavior. The implementation is the coder's choice (repo-rules, Outcome steps).
1. The force-push prompt shows the full branch name (feat/login, not login), without a refs/heads/ prefix.
   Proof: test `t7l_force_push_full_branch`.
2. If listing a commit's files fails, an error is shown and Loading ends with an empty list, without retrying in a loop.
   Proof: validator reads the code path.
3. A failed remote-tag lookup shows an error.
   Proof: validator reads the code path.
4. Updating a remote URL shows an error when the URL is missing or unreadable.
   Proof: validator reads the code path.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && test "$(cargo test --workspace --locked t7l_ 2>&1 | grep -cE 't7l_.* ok$')" -ge 1

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

### STEP 7M: Selections Survive a Refresh

* **Recommended Command:** `/goal`

```text
[STEP_ID]
STEP 7M: lists keep a valid, expected selection after refresh.

[SCOPE_LOCK]
ALLOWED_FILES:
  - src/components/utils/*
  - src/components/commitlist.rs
READ_ONLY_REFS:
  - neb: BUG-0054 (part). Replaces 7G
FORBIDDEN_SCOPE:
  - Any file not listed in ALLOWED_FILES. When the selected path is gone, selection still moves to its nearest neighbor as the existing tests expect.

[PRE_GATE]
true

[MUTATION_SPEC]
Outcome step: each item is a required behavior. The implementation is the coder's choice (repo-rules, Outcome steps).
1. With files a-b and a/c and a-b selected, refreshing the same list keeps a-b selected.
   Proof: test `t7m_refresh_keeps_dash_path`.
2. Shrinking the commit list clamps the selection to the last commit. An empty list is safe.
   Proof: test `t7m_commit_selection_clamps`.

[POST_GATE]
cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace --locked && test "$(cargo test --workspace --locked t7m_ 2>&1 | grep -cE 't7m_.* ok$')" -ge 2

[HALT_INVARIANT]
Halt immediately after POST_GATE exits 0 and report verification status.
```

---

## Phase 1: Parallel Subagent Component Audit (completed)

STEP 1A was completed by Gemini; findings are in `AUDIT_REPORT.md`. It has no step block (nothing to execute).
