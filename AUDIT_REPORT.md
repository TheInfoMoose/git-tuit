# `git-tuit` Component Audit & Remediation Report (`STEP 1A`)

**Audit Date:** 2026-11-20  
**Scope:** Full read-only repository audit + reference styling extraction from the Nebulous repo and the Orrery repo.  
**Pre-Gate Status:**
- Clean working tree: `git diff --name-only` is empty (`git status --short` showed only pre-existing untracked `STEPPED_IMPLEMENTATION_PLAN.md`).
- The Nebulous and Orrery reference repos were blocked inside the read-only subagent sandbox (container mount restriction + `read_file` permission timeout) and were inspected directly by the parent agent outside the sandbox.

---

## Executive Summary & Finding Counts

| Priority Tier | Total | STEP 2A | STEP 2B | STEP 2C | STEP 3A | STEP 3B | STEP 3C | STEP 4A | STEP 4B | STEP 4C |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **P0 — CI & Build Blockers** | **17** | 9 | 6 | 2 | 0 | 0 | 0 | 0 | 0 | 0 |
| **P1 — Security, Data-Loss & Production Panics** | **24** | 1 | 0 | 0 | 11 | 9 | 3 | 0 | 0 | 0 |
| **P2 — Usability, Guardrails, Resize & Hardening** | **24** | 0 | 1 | 1 | 0 | 3 | 18 | 0 | 1 | 0 |
| **P3 — Styling & Design-Token Alignment** | **8** | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 5 | 1 |
| **Total** | **73** | **10** | **7** | **3** | **11** | **12** | **21** | **2** | **6** | **1** |

### Blockers & Decisions Required Before Phase 2–4

1. **Local Crate Cache Warmup (Phase 2 Blocker):**
   - Only **158 / 460** `Cargo.lock` packages exist in `~/.cargo/registry/cache/`; **302 crates are missing** (`cargo metadata --offline --locked` fails on `easy-cast`).
   - **Action needed:** Run `cargo fetch --locked` with network access before starting `STEP 2B`/`2C`.
   - **Optional host parity tools:** `rustup toolchain install 1.88`, `rustup target add x86_64-unknown-linux-musl`, `musl-tools`, `tombi` (`0.9.0`), `cargo-nextest`.
2. **`chrono` Unmaintained Advisory (`RUSTSEC-2026-0543`, STEP 2B Decision):**
   - `chrono 0.4.44` (`Cargo.toml:35, 76`, `Cargo.lock:343`) has no patched release (`patched = []`).
   - **Option A:** Migrate the 5 usage sites (`build.rs`, `src/components/commitlist.rs`, `src/components/utils/{logitems,mod}.rs`, `src/popups/file_revlog.rs`) to `jiff 0.2.32` (already in `Cargo.lock` via `gix-date` and `env_logger`).
   - **Option B:** Add an explicitly reviewed ignore entry for `RUSTSEC-2026-0543` in `deny.toml` (per STEP 2B halt invariant, requires explicit approval).
3. **`[SCOPE_LOCK]` Addenda Needed in `STEPPED_IMPLEMENTATION_PLAN.md`:**
   - **STEP 2B:** Add `.clippy.toml` (`allow-unwrap-in-tests = true`, `allow-expect-in-tests = true`, `allow-panic-in-tests = true` to unblock **>1,270** `#[cfg(test)]` diagnostics under `--all-targets`) and `Makefile`.
   - **STEP 3A:** Add `asyncgit/src/sync/remotes/{callbacks.rs,mod.rs,push.rs,tags.rs}`, `asyncgit/src/sync/utils.rs`, `asyncgit/src/sync/config.rs`, `asyncgit/src/sync/mod.rs`, `asyncgit/src/fetch_job.rs`, and `asyncgit/src/blame.rs`. Note `git2_hooks::create_hook` public signature choice (`try_create_hook` preserves semver).
   - **STEP 3B:** Add `src/components/cred.rs` (password buffer wipe + Esc cancel hygiene) and `src/popups/commit.rs` (`COMMIT_EDITMSG` symlink/`0o600` + `commit.template` `shellexpand::full` RCE/arbitrary-read).
   - **STEP 3C:** Add `src/components/{changes.rs,commitlist.rs,status_tree.rs,revision_files.rs,diff.rs,textinput.rs,syntax_text.rs}`, `src/components/commit_details/{mod.rs,details.rs}`, `src/components/utils/{statustree.rs,filetree.rs,scroll_vertical.rs,scroll_horizontal.rs,mod.rs}`, `src/queue.rs`, `src/keys/{key_list.rs,symbols.rs}`, `filetreelist/src/{filetreeitems.rs,filetree.rs}`, and optionally `vim_style_key_config.ron`.
   - **STEP 4B:** Expand beyond the initial 5 files to include core panes & popup renderers (`src/components/{diff.rs,commitlist.rs,revision_files.rs,syntax_text.rs,textinput.rs}`, `src/ui/scrollbar.rs`, `src/spinner.rs`, `src/tabs/{status.rs,revlog.rs,stashing.rs}`, `src/popups/{mod.rs,msg.rs,goto_line.rs,externaleditor.rs,help.rs,options.rs,remotelist.rs,submodules.rs,branchlist.rs,fuzzy_find.rs,log_search.rs,blame_file.rs,file_revlog.rs,taglist.rs,push.rs,pull.rs,fetch.rs,push_tags.rs,checkout_option.rs,reset.rs,commit.rs}`) so borders and titles do not render mixed `Plain`/`Thick` side-by-side.
4. **Phase 4 Visual Design Choices to Confirm:**
   - **BorderType:** Nebulous and Orrery both use `BorderType::Plain` (`Borders::ALL`, `┌─┐│└─┘`), *not* `BorderType::Rounded` (correcting the placeholder in `STEPPED_IMPLEMENTATION_PLAN.md:352`; preserves `src/gitui.rs:229` snapshot filter).
   - **Diff / Status Accent Policy:** Strict 7-color monochrome P3 Amber (`AMBER_BRIGHT` add / `AMBER_ALERT` delete) vs. Amber CRT chrome with warm phosphor-green diff additions (`#7FD962`) + `AMBER_ALERT` (`#FF5500`) deletions.
   - **Canvas Background:** Full-frame `BG_BLACK` (`#000000`) CRT paint (exact Nebulous/Orrery behavior) vs. transparent `Color::Reset` terminal background.
   - **Selection Background:** Exact `PANEL_BG` (`#0C0800`) vs. lifted dark amber (`#2A1A00` / `Rgb(42, 26, 0)`) for multi-column legibility in diffs/trees.

---

## Section 1: CI, Build & Supply-Chain Audit (Auditor 1)

### 1.1 Local Environment, Tooling & Offline Cache Status

| Check / Tool | Status | Exact Evidence |
|---|---|---|
| `rustc` / `cargo` | Installed | `rustc 1.99.1 (58cf8f9c3 2026-10-13)`, `cargo 1.99.1`, `clippy 0.1.99`, `rustfmt 1.10.0-stable` |
| Rustup toolchains | **Partial** | Only `stable-x86_64-unknown-linux-gnu` installed; **missing MSRV `1.88`** |
| Rustup targets | **Partial** | Only `x86_64-unknown-linux-gnu` installed; **missing `x86_64-unknown-linux-musl`** |
| Host CI tools | **Partial** | Installed: `cargo-deny 0.20.2`, `gpgsm 2.4.4`, `gpg 2.4.4`, `ssh-keygen`, `gcc 13.3.0`, `openssl`, `pkg-config 1.8.1`. Missing: `tombi`, `cargo-nextest`, `musl-gcc` (`musl-tools`), `typos` |
| `cargo fmt --all -- --check` | **PASS** | Exit `0` |
| `cargo metadata --offline --locked --no-deps` | **PASS** | Exit `0` (7 workspace crates) |
| `cargo metadata --offline --locked` | **BLOCKED** | Exit `101`: `error: no matching package named 'easy-cast' found` (`158 / 460` crates cached; **302 missing**) |
| `cargo deny --offline check` | **BLOCKED** | Fails at `cargo metadata` due to missing 302 crates; local RustSec advisory DB (updated 2026-11-20) cross-checked directly against `Cargo.lock` |
| `cargo clippy` / `cargo test` (`--offline`) | **BLOCKED** | Requires `cargo fetch --locked` first |

### 1.2 Workflow Failures (`STEP 2A`)

- **`P0-01` (`.github/workflows/ci.yml:3-5`):** Daily upstream `schedule: cron: "0 2 * * *"` runs unattended CI jobs on the fork. Remove `schedule:`; keep `push`/`pull_request` on `master`.
- **`P0-02` (`.github/workflows/ci.yml:15-100`):** `build` matrix targets `macos-latest`, `windows-latest`, `nightly` (`ci.yml:19-20`) with OS-specific steps (`ci.yml:34-36, 61-63, 83-99`). Restrict to `ubuntu-latest` with `rust: [stable, "1.88"]`.
- **`P0-03` (`.github/workflows/ci.yml:68-70`):** Runs `make clippy` (`cargo clippy --workspace --all-features`), skipping `#[cfg(test)]`/example/bench targets, omitting `-D warnings`, and enabling mutually exclusive `regex-fancy` + `regex-onig` (`Cargo.toml:84-86`). Replace with `cargo clippy --workspace --all-targets -- -D warnings`.
- **`P0-04` (`.github/workflows/ci.yml:101-157`):** `build-linux-musl` fails due to 4 bugs: (1) matrix includes `nightly` and `"1.88"` (`ci.yml:106`); (2) `ci.yml:138-142` runs `sudo apt-get -qq install musl-tools` and `gpgsm` without `apt-get update` or `-y`; (3) `ci.yml:127` runs `rustup target add` *before* `rustup override set` at `ci.yml:130`, adding `musl` to the wrong toolchain; (4) `ci.yml:156` runs `cargo install --path "." --force --locked` without `--target`, recompiling a glibc build. Scope to `stable`, run `sudo apt-get update && sudo apt-get install -y musl-tools gpgsm`, configure `musl` target cleanly, and remove/fix the install step.
- **`P0-05` (`.github/workflows/ci.yml:158-226, 227-265, 305-326, 327-341, 342-355`):** Five upstream-only jobs (`build-linux-arm` fetching 2018 GCC 8.2 from `armkeil.blob.core.windows.net`, `build-apple-x86`, `udeps`, `log-test`, and `test-homebrew` using archived `actions-rs/toolchain@v1` at `ci.yml:350`). Delete all 5 jobs.
- **`P0-06` (`.github/workflows/ci.yml:284`):** `linting` runs `cargo fmt -- --check` without `--all`, skipping workspace member crates (`asyncgit`, `filetreelist`, `git2-hooks`, etc.). Change to `cargo fmt --all -- --check`.
- **`P0-07` (`.github/workflows/nightly.yml:1-125`):** Upstream cron job (`0 3 * * *`) runs `make test` without `gpgsm` (`nightly.yml:38-40`), runs `apt-get install` without `update`/`-y` (`nightly.yml:51`), uses `ubuntu-22.04` (`nightly.yml:17`), and uploads to upstream `s3://gitui/nightly/` with missing secrets (`nightly.yml:10, 87-125`). Delete `nightly.yml`.
- **`P0-08` (`.github/workflows/brew.yml:1-23`):** Upstream Homebrew formula bumper using missing `secrets.BREW_TOKEN`. Delete `brew.yml`.
- **`P0-09` (`.github/workflows/cd.yml:1-134`):** Tag-triggered release workflow includes `macos-latest`, `windows-latest`, `ubuntu-22.04` (`cd.yml:17`), runs `make test` without `gpgsm` (`cd.yml:46-48`), runs `apt-get -qq install musl-tools` without `update`/`-y` (`cd.yml:59`), bumps Homebrew with missing `BREW_TOKEN` (`cd.yml:126-134`), and fails `gh-release` on `workflow_dispatch`. Trim to Linux-only `ubuntu-latest` or disable tag release until needed.
- **`P1-01` (`.github/workflows/ci.yml:297-299`):** `cargo install --locked cargo-deny` builds unpinned latest from source on every run. Replace with pinned prebuilt `taiki-e/install-action@cargo-deny` (or `EmbarkStudios/cargo-deny-action@v2`).
- **`P2-01` (`.github/workflows/ci.yml:25,32,39,109,116,270,277`, `.github/dependabot.yml:6`, `Cargo.toml:7,17-24`):** Cache key mismatch (`shared-key` vs `key`), redundant `rustup override set` fighting `rust-toolchain.toml`, Node 20 `actions/checkout@v4` check, daily `dependabot.yml` cadence, member `Cargo.toml`s omitting `rust-version = "1.88"`, and `invalidstring` omitted from `[workspace].members`.

### 1.3 `cargo-deny`, Advisories, `tombi`, MSRV & Clippy (`STEP 2B`)

#### Active RustSec Advisories Against `Cargo.lock` (Local DB 2026-11-20)

| ID | Advisory | Crate (`Cargo.lock` / Manifest) | Path | Remediation |
|---|---|---|---|---|
| **`P0-10`** | `RUSTSEC-2026-0418` (**VULN**) + `RUSTSEC-2026-0544` (`1.1.25..1.1.32` heap overflow) | `libz-sys 1.1.21` (`Cargo.lock:2470`) | `asyncgit -> git2 0.21.0 -> libgit2-sys -> libz-sys` | `cargo update -p libz-sys` to **`>= 1.1.33`** (must skip `1.1.31`/`1.1.32`). |
| **`P0-11`** | `RUSTSEC-2026-0543` (**unmaintained**) | `chrono 0.4.44` (`Cargo.toml:35,76`, `Cargo.lock:343`) | Direct dep + build-dep | Migrate 5 files to `jiff 0.2.32` (already in `Cargo.lock`) OR get explicit approval to ignore in `deny.toml`. |
| **`P0-12`** | `RUSTSEC-2026-0253` (**unsound**) | `lru 0.16.3` (`Cargo.lock:2530`) | `ratatui 0.30.0 -> ratatui-core 0.1.0 -> lru` | `cargo update -p ratatui-core` to `0.1.2+` (upgrades `lru` to `^0.18`; keeps `rust-version = "1.88.0"`). |
| **`P0-12`** | `RUSTSEC-2026-0306` (**unsound**) (+ check `RUSTSEC-2026-0574..0582` `heapless 0.8.0` at `Cargo.lock:2015`) | `faster-hex 0.10.0` (`Cargo.lock:862`) | `gix 0.84.0 -> gix-hash 0.25.1 -> faster-hex` | `cargo update -p faster-hex` (`>= 0.10.1`); verify `cargo tree -i heapless` post-fetch. |
| **`P0-12`** | `RUSTSEC-2026-0486` (**unsound**) | `rustix 1.1.4` (`Cargo.lock:3284`) | `crossterm`, `tempfile`, `which`, `gix-index` | `cargo update -p rustix` to `1.1.6+`. |
| **`P0-12`** | `RUSTSEC-2026-0097` (**unsound**) | `rand 0.8.5` (`Cargo.lock:3056`) | `gh-emoji 1.0.8 -> phf -> phf_generator -> rand` | `cargo update -p rand` to `0.8.6+`. |
| **`P0-13`** | `RUSTSEC-2026-0205` (**unsound**) | `scc 2.3.4` (`asyncgit/Cargo.toml:41`, `Cargo.lock:3318`) | `asyncgit [dev-deps] -> serial_test 3.3.1 -> scc` | Update `serial_test` so `scc >= 3.8.4` (or replace dev-dep if unpatched). |
| **`P0-13`** | `RUSTSEC-2026-0441` (**unmaintained**) | `pin-utils 0.1.0` (`asyncgit/Cargo.toml:41`, `Cargo.lock:2942`) | `asyncgit [dev-deps] -> serial_test 3.3.1 -> futures-util` | Set `serial_test = { version = "...", default-features = false }` in `asyncgit/Cargo.toml:41` (`asyncgit` uses sync `#[serial]` only: `sign.rs:414`, `cred.rs:206`, `revlog.rs:332`). |

*(Note: `RUSTSEC-2025-0141` on `bincode 1.3.3` via `syntect` is already ignored at `deny.toml:18`.)*

#### `deny.toml`, Clippy `--all-targets`, `tombi` & `build.rs`

- **`P0-14` (`deny.toml:15, 26, 29, 31, 35, 39, 43, 46, 53, 59, 62`):** Legacy `[advisories] version = 2` and `{ name = "..." }` `skip-tree` syntax (deprecated/rejected in `cargo-deny 0.20.2`), plus 3 stale `skip-tree` entries (`unicode-truncate`, `redox_syscall`, `sha1-checked`). Remove `version = 2`, convert to `{ crate = "..." }`, remove stale entries, and re-verify `cargo deny check bans licenses` post-fetch (`crossterm 0.28.1/0.29.0`, `sha2`, `digest`, `fancy-regex`, `syn`, `bitflags`, `terminfo`).
- **`P0-15` (`.clippy.toml:1-2`, `src/main.rs:43-50`, `asyncgit/src/lib.rs:21-31`, `asyncgit/src/sync/mod.rs:4`, `asyncgit/src/asyncjob/mod.rs:3`, `filetreelist/src/lib.rs:11-18`, `git2-hooks/src/lib.rs:21`, `asyncgit/src/revlog.rs:370, 403`):** Workspace crates enforce `#![deny(clippy::all, clippy::perf, clippy::nursery, clippy::pedantic, clippy::unwrap_used, clippy::panic)]` (plus `expect_used`). Static scan found **1,268 `.unwrap()`, 14 `.expect()`, and 7 `panic!()`** in `#[cfg(test)]` modules with zero test allows (`src/`: 75/8/0, `asyncgit/`: 1,129/6/5, `filetreelist/`: 64/0/0, `git2-hooks/`: 65/0/2), plus `clippy::unit_cmp` at `asyncgit/src/revlog.rs:370, 403`. Add `allow-unwrap-in-tests = true`, `allow-expect-in-tests = true`, and `allow-panic-in-tests = true` to `.clippy.toml` and fix source/test Clippy diagnostics without blanket `#[allow(...)]`.
- **`tombi` / MSRV `1.88` (Pass):** Manual inspection of all 14 `.toml` files + `tombi.toml:1-5` shows valid LF formatting, max line width `139 <= 220`, alphabetical dependency tables, and zero TOML 1.1 inline tables; `d7214ecc` (`git2` `"unstable-sha256"`) adds zero extra crates. Verify with `tombi format --check` when available.
- **`P2-02` (`deny.toml` + `build.rs:8-10, 24, 29-31, 47`):** Add `[sources]` (`unknown-registry = "deny"`, `unknown-git = "deny"`, crates.io allowlist) and Linux `[graph] targets` to `deny.toml`; note `build.rs` slice/unwrap hazards (`commit[..7]`, `SOURCE_DATE_EPOCH.unwrap()`, unconditional `cargo:warning`).

### 1.4 `Makefile` & Test Harness Parity (`STEP 2C`)

- **`P0-16` (`Makefile:2, 21-43, 49-54, 61-62, 69-110`, `.cargo/config.toml:1-8`):** `test` (`Makefile:94`) and `test-linux-musl` (`Makefile:62`) hard-require `cargo-nextest`; `fmt` (`Makefile:97`) omits `--all`; `clippy` (`Makefile:100`) runs `--all-features` instead of `--all-targets -- -D warnings`. Align `Makefile` targets with Linux CI (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` fallback) and clean `.PHONY`/dead targets.
- **`P0-17` (`asyncgit/src/revlog.rs:391-401`, `asyncgit/src/sync/sign.rs:755, 870, 907, 991`):** Process-wide env var mutation hazards under multi-threaded `cargo test --workspace`: `test_env_variables` (`revlog.rs:391`) mutates `GIT_DIR` under `#[serial]` while ~50 non-`#[serial]` `asyncgit` tests run concurrently in the same process (and leaks on panic); `test_x509_sign_and_verify_e2e` and `test_openpgp_sign_and_verify_e2e` mutate `GNUPGHOME` without an RAII restore guard (unlike `sign.rs:1078-1092`). Wrap in RAII guards and re-run full `cargo test --workspace` after `cargo fetch --locked`.

---

## Section 2: Security & Git-Execution Audit (Auditor 2)

Strictly line-by-line audit of `asyncgit/src/sync/{sign.rs, cred.rs, hooks.rs, config.rs, utils.rs, remotes/*}`, `git2-hooks/src/*`, `src/popups/externaleditor.rs`, `src/clipboard.rs`, `src/args.rs`, plus direct callers (`src/components/cred.rs`, `src/popups/commit.rs`, `asyncgit/src/{push,pull,push_tags,fetch_job,lib}.rs`, `src/ui/syntax_text.rs`). No hardcoded secrets were found. Five production `.unwrap()`/`.expect()` sites exist in audited scope (`git2-hooks/src/lib.rs` x4, `src/popups/externaleditor.rs` x1).

### 2.1 `asyncgit` & `git2-hooks` Findings (`STEP 3A`)

- **`P1-02` (`asyncgit/src/sync/cred.rs:100-115, 130-145, 160-175`) — Untrusted `.git/config` `credential.helper` Shell RCE & Newline Injection:**
  - `helper.config(&repo.config())` loads repo-local `.git/config` (`ConfigLevel::Local`/`Worktree`) and executes `sh -c "<cmd> get"` (`git2 0.21.0`). Repo-local `credential.username` also injects `\r`/`\n` into helper stdin (CVE-2020-5260 class).
  - **Fix:** Use `git2::Config::open_default()` exclusively (global/XDG/system only); parse `url` via `url::Url::parse`, reject `\r`/`\n`/`\0` in host/path/username, strip embedded userinfo before `CredentialHelper::new`, and set clean username via `helper.username(...)`.
- **`P1-03` (`asyncgit/src/sync/cred.rs:50, 65, 85, 94-99, 154-159`, `asyncgit/src/sync/remotes/mod.rs:302-334`, `callbacks.rs:179-220`, `push.rs`, `tags.rs`, `hooks.rs`) — Cross-Host / Cross-Remote / HTTP Redirect Credential Exfiltration:**
  - `extract_username_password` and `extract_username_password_for_push` ignore `remote.pushurl()`. `url.starts_with("http")` matches non-HTTP URLs and plaintext `http://`. `fetch_all` (`mod.rs:319-325`) broadcasts default remote's `basic_credential` to *all* remotes. `Callbacks::credentials` returns `Cred::userpass_plaintext` for any redirected/challenged URL without origin validation.
  - **Fix (`STEP 3A` scope addendum):** Prefer `remote.pushurl()` when checking/extracting push creds; validate scheme with `url::Url::parse`; add `Callbacks::with_expected_url(self, url: &str) -> Self` enforcing `(scheme, host, port)` match (blocking cross-host & HTTPS->HTTP downgrades); resolve per-remote creds in `fetch_all`.
- **`P1-04` (`asyncgit/src/sync/cred.rs:16-22`, `asyncgit/src/sync/remotes/callbacks.rs:185-187`) — Plaintext Password/Token Leaks in `Debug` & Logs:**
  - `#[derive(Debug)]` on `BasicAuthCredential` (and transitively `PushRequest`/`FetchRequest`/`PushTagsRequest` in `push.rs:18`, `pull.rs:17`, `push_tags.rs:17`) exposes plaintext `password`. `callbacks.rs:186` logs raw `url` (including userinfo secrets) to `LevelFilter::Trace` `gitui.log`.
  - **Fix:** Manual `fmt::Debug` redacting `password` + `Drop` zeroing `password` bytes on `BasicAuthCredential` (note `zeroize` crate would touch `asyncgit/Cargo.toml`/`Cargo.lock` if volatile zeroing is desired); redact userinfo from `url` before `log::debug!` in `callbacks.rs`.
- **`P1-05` (`git2-hooks/src/hookspath.rs:42-95, 103-133, 296-310`) — Arbitrary `$ENV_VAR` Expansion, Hook Name Traversal & Directory/World-Writable Execution:**
  - `shellexpand::full(...)` expands `$SECRET` env vars from repo-local `core.hooksPath`; unvalidated `hook` allows `../../bin/sh`; relative `core.hooksPath` escapes worktree; `is_executable` returns `true` for directories (`0o755 & 0o111 != 0`) and symlinks to world-writable files.
  - **Fix:** Replace `shellexpand::full` with `shellexpand::tilde` (keeping `HooksError::ShellExpand`); validate `hook` is a single `Component::Normal`; canonicalize relative `core.hooksPath` and assert containment within canonical `pwd` (and `other_paths` within repo/gitdir); require `metadata.is_file()`, `mode & 0o111 != 0`, and `mode & 0o002 == 0`.
- **`P1-06` (`git2-hooks/src/hookspath.rs:208-256`) — Missing `--` in Unix `ENOEXEC` `/bin/sh` Fallback & Windows `sh -c` Interpolation:**
  - `run_command(sh_command().arg(&hook))` omits `--` (hook starting with `-c` injects shell flags). Windows branch builds single-quoted `sh -c` string.
  - **Fix:** Pass `sh_command().arg("--").arg(&hook)` on Unix and `sh_command().arg("-c").arg("\"$0\" \"$@\"").arg(&hook)` on Windows.
- **`P1-07` (`git2-hooks/src/lib.rs:195-204, 297-330`) — Insecure `COMMIT_EDITMSG` Temp File Handling:**
  - `File::create` and `File::open` follow symlinks (`COMMIT_EDITMSG -> ~/.bashrc`), create with `0o644`, leak temp files on error/panic, read unbounded bytes (`/dev/zero` DoS), and `hooks_prepare_commit_msg` (`lib.rs:300`) corrupts non-UTF-8 paths via `.to_string_lossy()`.
  - **Fix:** Verify `symlink_metadata` is not symlink/non-regular, remove existing regular file, recreate atomically with `OpenOptions::new().write(true).create_new(true).mode(0o600)`, attach RAII cleanup guard, verify non-symlink + bound `Read::take(MAX_COMMIT_MSG_BYTES)` on read-back, and use `run_hook_os_str` in `hooks_prepare_commit_msg`.
- **`P1-08` (`git2-hooks/src/lib.rs:150-176`) — Production `.unwrap()` Panics (x4) & Unsafe `chmod` Subprocess:**
  - `pub fn create_hook` / `create_hook_in_path` contain 4 `.unwrap()` calls (`lib.rs:155, 165, 165, 174`) and run `Command::new("chmod").arg("+x").arg(path)` (PATH hijack, missing `--`, `0o755`, unchecked status).
  - **Fix:** Replace `chmod` with `std::fs::set_permissions` (`0o700`) / `OpenOptionsExt::mode(0o700)` and propagate `Result<PathBuf>` via `try_create_hook` (preserving `create_hook` semver or updating all callers if breaking signature is approved). Clean duplicate test helper at `asyncgit/src/sync/hooks.rs:332-337`.
- **`P1-09` (`asyncgit/src/sync/sign.rs:152-164, 195-197, 206-249, 266-274, 344, 352-362`) — Signing Program Validation, Argument Injection & Inline Key Exposure:**
  - `gpg.*program` allows relative paths (`./evil`) from local config and spawns bare names via unsanitized `PATH` with CWD in repo; `SSHSign` only creates temp file for `ssh-` keys (exposing `ecdsa-sha2-*`/`sk-*` or misconfigured private keys in `/proc/*/cmdline` & `log::trace!`); `GPGSign`/`SSHSign` omit leading-`-`/`\n`/`\0` validation on `signingKey` and `--`; `.display().to_string()` corrupts non-UTF-8 `$TMPDIR` paths.
  - **Fix:** Reject empty/NUL/relative-with-separator/`..` signing programs and resolve bare names via absolute-only `PATH` before spawn; treat `key::` and all SSH pubkey prefixes as temp-file keys (`0o600` explicit, reject `PRIVATE KEY` & `\n`, store `PathBuf`); validate `signingKey` against leading `-`/NUL/newlines; pass `--` / `--local-user=`; redact args from `log::trace!`; match `file_name() == "ssh-keygen"` at `sign.rs:358`.
- **`P1-10` (`asyncgit/src/sync/hooks.rs:101-106, 226-278`, `git2-hooks/src/lib.rs:90-108`) — `pre-push` Hook Stdin Refspec Injection:**
  - Unvalidated `branch.<name>.merge` / ref names containing `\n` or whitespace forge additional ref lines on `pre-push` stdin.
  - **Fix:** Validate `local_ref` and `remote_ref` via `git2::Reference::is_valid_name` / whitespace+control rejection in `hooks.rs` and `hooks_pre_push`.
- **`P1-11` (`asyncgit/src/sync/utils.rs:182-194`, caller `discard_tracked.rs:40`) — Path Traversal & Symlink Overwrite in `repo_write_file` (`[SCOPE_LOCK]` Addendum):**
  - Joins unvalidated `file` onto `work_dir` and calls `File::create` (truncates symlink targets outside repo).
  - **Fix:** Reject absolute paths / `..`, canonicalize `work_dir` and parent dir + assert `starts_with`, check `symlink_metadata`, and use atomic temp-file + rename in same dir.
- **`P1-12` (`asyncgit/src/sync/config.rs:111-140`, `asyncgit/src/sync/mod.rs`) — Missing Trusted Global-Only Config Reader (`[SCOPE_LOCK]` Addendum):**
  - `config.rs` exposes only merged `repo.config()`, forcing `externaleditor.rs` (`core.editor`) and `commit.rs` (`commit.template`) to trust repo-local `.git/config`.
  - **Fix:** Export `pub fn get_global_config_string(key: &str) -> Result<Option<String>>` backed by `git2::Config::open_default()`.

### 2.2 External Editor, Clipboard, CLI Args & Credential/Commit Callers (`STEP 3B`)

- **`P1-13` (`src/popups/externaleditor.rs:69-78, 112-116`) — Untrusted Repo-Local `core.editor` RCE & CWD Hijack:**
  - When `GIT_EDITOR` is unset, repo-local `core.editor` overrides `$VISUAL`/`$EDITOR` and executes arbitrary commands with `.current_dir(work_dir)` via raw `PATH`.
  - **Fix:** Read `core.editor` via `get_global_config_string` (`P1-12`), reject relative executables with separators/`..`, resolve bare binary via sanitized absolute-only `PATH` before `.current_dir(work_dir)`, and filter empty env vars.
- **`P1-14` (`src/popups/externaleditor.rs:48-62, 107-116`) — Path Traversal, Symlink Escape & Missing `--` Option Terminator:**
  - Opens relative/absolute paths and symlinks outside `work_dir`/`git_dir` without canonicalization and appends filename directly to `args` without `--` (allowing `-` flag / `+!cmd` injection).
  - **Fix:** Canonicalize `work_dir`, `git_dir`, and target `path`; require `is_file()` inside `canonical_work_dir` or `canonical_git_dir`; insert `--` before target path.
- **`P1-15` (`src/popups/externaleditor.rs:80-108`) — Broken Command Quote/Whitespace Splitting:**
  - Only parses leading `"` and splits remainder on whitespace, breaking single quotes, escapes, tabs, and paths with spaces.
  - **Fix:** Implement deterministic POSIX-style tokenizer (`parse_editor_command`) rejecting unmatched quotes, NULs, and empty commands without invoking a shell; add unit tests.
- **`P1-16` (`src/popups/externaleditor.rs:64-67`) — Production `.expect()` Panic Inside `defer!` Drop Guard:**
  - `io::stdout().execute(EnterAlternateScreen).expect("reset terminal")` inside `defer!` risks double-panic `SIGABRT` on PTY/stdout error. Log error instead.
- **`P1-17` (`src/clipboard.rs:13-17, 119`) — PATH Hijacking & Unsafe Fallback Execution:**
  - `which(command).ok().unwrap_or_else(|| PathBuf::from(command))` searches `.`/relative `PATH` entries and falls back to executing relative `./wl-copy`/`./xclip`/`./xsel` in repo CWD.
  - **Fix:** Use `which::which_in` against strictly absolute `PATH` directories, fail closed if missing, check `!val.is_empty()` for `WAYLAND_DISPLAY`, and log failure before OSC 52 fallback.
- **`P1-18` (`src/args.rs:71-82`, `src/ui/style.rs:293-324`) — `-t`/`--theme` Path Traversal & Arbitrary Read/Overwrite:**
  - `-t /etc/passwd` or `-t ../../x.ron` escapes `~/.config/gitui/` via `PathBuf::join` and triggers `File::create` rewrite if matching legacy `Theme`.
  - **Fix:** Validate `-t` argument is a single `Component::Normal` filename (no `/`, `\`, `..`, NUL, or empty string), preserving dotfile symlinks inside `~/.config/gitui/`.
- **`P1-19` (`src/args.rs:195-227`) — World-Readable (`0o644`) Trace Log & Symlink Overwrite:**
  - `setup_logging` uses `File::create` (umask `0o644`, follows symlinks) at `LevelFilter::Trace`.
  - **Fix:** Reject symlinks/non-regular files via `symlink_metadata`, create with `OpenOptions::new().write(true).create(true).truncate(true).mode(0o600)`, and enforce `0o700` on newly created cache dir (`args.rs:220`).
- **`P1-20` (`src/components/cred.rs:63-68, 101-108, 130-132, 147-161`) — `CredComponent::hide()` Retains Password & Hijacks Next Auth Prompt (`[SCOPE_LOCK]` Addendum):**
  - `Esc` hides `CredComponent` without hiding/clearing `input_username` or `input_password`. On next open, `input_password` draws on top while `input_username` captures keystrokes (user types password visibly into username!).
  - **Fix:** Hide and `.clear()` both `input_username` and `input_password` inside `CredComponent::hide()` and after submission.
- **`P1-21` (`src/popups/commit.rs:164-200, 400-424`) — `COMMIT_EDITMSG` Symlink/`0o644` & Untrusted `commit.template` `shellexpand::full()` RCE/File Read (`[SCOPE_LOCK]` Addendum):**
  - `CommitPopup` creates `<git_dir>/COMMIT_EDITMSG` with `File::create` (`0o644`, follows symlinks, leaks file, unbounded read-back) and reads repo-local `commit.template` through `shellexpand::full()` + `read_to_string()`, allowing an untrusted repo to exfiltrate arbitrary local files (`~/.ssh/id_rsa`) or `$ENV` vars into the commit box.
  - **Fix:** Apply `P1-07` atomic `0o600` non-symlink temp-file handling; read `commit.template` with `shellexpand::tilde` (or only from trusted global config) and enforce bounded regular-file read.
- **`P2-03` (`src/clipboard.rs:74-84, 92, 114`):** Cap OSC 52 raw payload (e.g. `74_994` bytes) and call `out.flush()?`.
- **`P2-04` (`src/args.rs:52-69, 177-192`, `asyncgit/src/sync/utils.rs:28-41`):** Reject empty/NUL `-d`/`-w` values and verify/canonicalize `-w` directory.
- **`P2-05` (`src/popups/externaleditor.rs:69-118`, `asyncgit/src/lib.rs:141-147`, `src/ui/syntax_text.rs:93-95`):** Check editor `ExitStatus::success()` (`:cq` aborts cleanly) and ignore empty `GIT_EDITOR=""`; note side findings on `trace-libgit` sensitive header logging and unvalidated `syntax` theme filename join in `syntax_text.rs`.

---

## Section 3: Core TUI, Tabs & Components Audit (Auditor 3)

Audit of `src/app.rs`, `src/tabs/*`, `src/components/**`, `src/keys/*`, `src/cmdbar.rs`, `src/strings.rs`, `src/queue.rs`, and `filetreelist/src/*`.

### 3.1 Crashes, Data Loss & Production Panics (`STEP 3C`)

- **`P1-22` (`src/app.rs:1060-1086`, cross-ref `branchlist.rs:680-688`, `branch/mod.rs:156`) — Wrong Remote Branch Deleted When Branch Contains `/`:**
  - `Action::DeleteRemoteBranch("refs/remotes/<remote>/<branch>")` calls `branch_ref.rsplit('/').next()` (`app.rs:1066`), discarding `<remote>` and stripping everything except the final slash segment (`refs/remotes/upstream/feat/login` -> deletes `login` on `origin`!).
  - **Fix:** Strip `refs/remotes/`, match longest configured remote + `/`, separate `(remote, full_branch_name)`, push deletion explicitly to `remote` (requires `STEP 3A` remote parameter on `PushPopup` / `push.rs`), and show `ShowErrorMsg` if parsing fails.
- **`P1-23` (`src/app.rs:381-382, 765, 825-827, 845-848, 966-980, 999-1001, 1048-1056`, `src/components/changes.rs:274`, `src/tabs/stashlist.rs:42`, `src/tabs/stashing.rs:264`, `src/popups/{push.rs:321, push_tags.rs:243, pull.rs:268, commit.rs:583}`) — Fatal `?` Propagation Exits TUI (`src/gitui.rs:123, 126, 135`):**
  - Uncaught `?` inside `App::event` / `update` / `process_internal_event` / `process_confirmed_action` crashes the app on `RewordCommit` (dirty repo `bail!`), `Push`/`PushTags` (no remote), `delete_tag` (no remote after local delete), `ResetHunk`/`ResetLines`, `stage_remove_all`, `get_stashes`, `untracked_files_config`, `amend`, and `get_files_changes` (leaves `input.set_polling(false)` stuck).
  - **Fix:** Replace `?` with `try_or_popup!` / `InternalEvent::ShowErrorMsg`; in `delete_tag`, always refresh `tags_popup` and prompt `DeleteRemoteTag` only if `get_default_remote` is `Ok`; move `get_files_changes()` inside `result` in `app.rs:381`; only push `StatusLastFileMoved` in `changes.rs:276` on `Ok`.
- **`P1-24` — Production Panics (Underflow, OOB Index, `.expect()`) Across Tree, CommitList & Popups:**
  - `filetreelist/src/filetreeitems.rs:218, 258`: `current_folder.1 - 1` `u8` underflow panic when `indent == 0` (also missing `else if` at `:258`).
  - `src/components/commitlist.rs:277, 711`: Unchecked `self.commits[self.selection]` panics on empty list (`select_next_highlight()` at `:681-688`) or shrunk list.
  - `src/components/utils/statustree.rs:231-237, 249, 258`: Checks `current_index == 0` instead of `cur_index_find == 0` -> `usize` underflow panic; unchecked `available_selections[...]` index.
  - `src/components/status_tree.rs:178` & `src/components/utils/filetree.rs:213`: `.expect("invalid path.")` in `draw()` / `push_dirs()`.
  - `src/popups/taglist.rs:391`: `tags.len() - 1` underflow on empty tag list.
  - `src/popups/blame_file.rs:329, 706, 788`: `lines().len() - 1` underflow and `lines()[selected]` OOB on 0-line files.
  - `src/popups/branchlist.rs:590`, `src/popups/confirm.rs:194`, `src/popups/remotelist.rs:416-419`: `.expect("Could not get status")`, `.expect("There was no /...")`, and `>=` off-by-one in `valid_selection()` causing OOB panic.

### 3.2 Usability, Focus, Resize & Error-Surfacing Findings (High/Medium/Low -> `STEP 3C` & `STEP 4B`)

- **`P2-13` (High, `STEP 3C` — `components/status_tree.rs:320,485`, `components/revision_files.rs:488,496`, `tabs/{files.rs:66-101, status.rs:141-142}`):** Calling `self.hide()` before opening `BlameFilePopup`/`FileRevlogPopup` makes `FilesTab`/`Status`/`Stashing` pane vanish permanently when popup closes (until 5s ticker or forever in `-w` mode). Remove `self.hide()` back-channel or emit `NeedsUpdate::ALL` / restore visibility on `PopupStackPop`.
- **`P2-14` (High, `STEP 3C` — `tabs/revlog.rs:128-165, 280-290`, `tabs/files.rs:50-59`):** `Revlog` gates `CommitFilter` behind `self.visible`, permanently locking search in `Searching` if user switches tabs; `FilesTab` drops `SyntaxHighlighting(Done)` when hidden, sticking at `(0%)`.
- **`P2-15` (High, `STEP 3C` — `filetreelist/src/{filetree.rs:70-75,110-121,168-197, filetreeitems.rs:140-151}`, `app.rs:885-892`, `components/revision_files.rs:246-251`):** `FileTree::select_file` sets `selection = None` on miss (permanently breaking keyboard nav in `FilesTab`) and skips `show_element()` when re-selecting same file after `collapse_but_root()`; `collapse(..., true)` flips next sibling icon before checking `starts_with`; missing `calc_visual_selection()` breaks scrollbar. Route `FuzzyFinderTarget::Files` only to the active target.
- **`P2-16` (Medium, `STEP 3C` — `components/commit_details/{details.rs:184-266, mod.rs:147-168}`, `components/utils/{scroll_vertical.rs:38-43,85-151, scroll_horizontal.rs:52-117}`, `components/syntax_text.rs:98-137,218-222`, `strings.rs:426-439`):** At 80x24, `DetailsComponent` uses `[Length(8), Min(10)]` inside an ~8-row pane, collapsing a sub-panel to 0 rows and clipping tags when `committer.is_some()`; `CompareDetailsComponent` leaves 1 file row at 60x16; scroll offsets stay out-of-bounds after terminal growth; `move_top` inverts direction at `visual_height == 0`; `SyntaxTextComponent` retains prior file's scroll offset; `ellipsis_trim_start` overflows `width < 3`.
- **`P2-17` (High, `STEP 3C` — `tabs/stashing.rs:72-80, 229-245, 262-273`):** `Stashing` file tree ignores `stash_untracked` and `keep_index` toggles (`//TODO: support options`) and resets `stash_untracked` on tab switch.
- **`P2-18` (High, `STEP 3C` — `queue.rs:23-35`, `tabs/status.rs:384-428, 612-624, 874-899`, `popups/branchlist.rs:321-342, 506`, `popups/submodules.rs:364`):** `NeedsUpdate::REMOTES = 0b1001` collides with `ALL | BRANCHES` (fix to `0b10000`); `Status` never calls `check_remotes()` on update, blocks `Fetch` without upstream, and shows `Push` enabled on detached HEAD; `BranchListPopup` skips `AsyncGitNotification::Fetch`, only removes first `*/HEAD` ref, and both `branchlist.rs:506` and `submodules.rs:364` discard `.unicode_truncate()`'s return slice!
- **`P2-19` (High, `STEP 3C` — `vim_style_key_config.ron:18-19, 29`, `keys/key_list.rs:207`, `components/textinput.rs:368-396`):** `vim_style_key_config.ron` binds `status_reset_item` to `Shift+U`, colliding with default `undo_commit` (`Shift+U`); `TextInputComponent` swallows `Ctrl+n`/`Ctrl+p` (`popup_down`/`popup_up` in vim config) even in `InputType::Singleline`.
- **`P2-20` (Medium, `STEP 3C` — `components/utils/{statustree.rs:55-70,192-207, mod.rs:49-69}`, `components/commitlist.rs:93-96, 209-215, 390-406, 555-557`):** `StatusTree` calls `binary_search_by` on unsorted items (`'-' < '/'`), jumping selection on refresh; `CommitList` fails to clamp `selection` on shrink, stores slice-relative `marked` indices across 1200-commit boundaries, and sums UTF-8 byte `.len()` / char counts instead of Unicode display widths.
- **`P2-22` (Medium, `STEP 3C` — `components/revision_files.rs:133, 260-265, 431-530`, `tabs/{files.rs:36, status.rs:467-480, revlog.rs:464-474}`, `app.rs:182-189, 252, 296, 634`, `components/diff.rs:276`, `keys/{key_list.rs:240-242, symbols.rs:58}`, `components/status_tree.rs:147`, `components/textinput.rs:206, 274-282, 682, 706, 724`, `keys/key_config.rs:99, 121-128`, `strings.rs:693, 759-764, 1328, 1772`):** `AsyncTreeFilesJob` errors and unborn-HEAD `FilesTab` stick at `"Loading ..."`; invalid persisted `current_tab >= 5` fails every `App::draw` with `bail!("unknown tab")`; `Focus::Diff` gets stranded empty after staging last hunk; `Revlog` `Esc` doesn't close `commit_details`; `RevisionFiles` loses focus into empty viewer and `e` switches tab behind `RevisionFilesPopup`; `Diff` `Shift+Down` overshoots by 1 line; key/symbol parse errors go unlogged; `Typechange` renders `' '` instead of `'T'`; `TextInput` `Tab` inserts `\t` in singleline/password inputs and counts bytes instead of chars; `quit` hint omits `q`, `diff_home_end` formats `move_up`/`move_down`.
- **`P2-24` (High, `STEP 4B` — `src/cmdbar.rs:37, 82-114, 133-198`, `src/app.rs:351-356, 1160-1228`, `src/popups/{confirm.rs:38, reset.rs:160, checkout_option.rs:130}`):** `CommandBar` inserts leading `LineBreak` when `line_width == 0` on narrow terminals (rendering blank row 0); hardcodes `"more [.]"`/`"less [.]"`; drops `r.x +`; fails to clear `expanded` when `!expandable`; doesn't cap height against terminal height; `cmd_bar_toggle` blocked in popups; top tab bar clips at `<= 60` cols; confirmation popups clip details at fixed `55x5` / `50%x20%`.

---

## Section 4: Popups & Interactive Flows Audit (Auditor 4)

All 32 files in `src/popups/`, `src/popup_stack.rs`, `src/queue.rs`, and `src/strings.rs` were audited.

### 4.1 Destructive Actions Confirmation Matrix (All 24 Actions)

| # | Action | Trigger (`file:line`) | Has Confirm? | Confirm Details / Target Named? | Gap / Risk |
|---|---|---|---|---|---|
| 1 | **Reset repo (Soft / Mixed / Hard)** | `src/tabs/revlog.rs:544-557` (`R`), `src/popups/branchlist.rs:197-204` (`s`) -> `src/popups/reset.rs:112-138, 149, 241-243` | **Partial (picker only; NO confirm for Hard)** | Current branch, full SHA, mode, emoji desc. | **High (`P2-06`):** `ResetPopup::open()` does NOT reset `self.kind = Soft` (remembers `Hard` even after `Esc`!). `move_up` wraps `Soft -> Hard`. Missing `NeedsUpdate::ALL` on reset. |
| 2 | **Checkout branch + Discard all changes (`reset --hard HEAD`)** | `src/popups/branchlist.rs:580-614` -> `src/popups/checkout_option.rs:80-116, 211-217` | **Partial (picker only; NO confirm)** | Target branch + `Discard all local changes`. | **High (`P2-06`):** `open()` remembers `DiscardAllLocalChagnes` across opens! Runs `discard_status()` (`status.rs:289-296`, `ResetType::Hard` destroying staged+unstaged) *before* `checkout()` (if checkout fails, changes are gone). |
| 3 | **Discard working-tree file / folder / untracked** | `src/components/changes.rs:149-160, 279-284` (`D`), `diff.rs:618-624, 915-917` -> `confirm.rs:131-134` | **Yes** | **NO target named** (`"confirm file reset?"`, `strings.rs:146-148, 202-204`). | **High (`P2-07`):** Ignores `path`; `sync::reset_workdir` (`reset.rs:30-37`) uses `.remove_untracked(true).force()`, **permanently deleting untracked files/dirs** with zero warning. |
| 4 | **Discard diff hunk** | `src/components/diff.rs:547-560, 919` (`D`) -> `confirm.rs:145-148` | **Yes** | **NO target named** (`strings.rs:235-239`). | **Medium (`P2-07` / `P1-23`):** Omits file path/hunk; `app.rs:966-973` uses bare `?` (crashes TUI on error). |
| 5 | **Discard diff lines** | `src/components/diff.rs:562-569, 931-942` (`d`) -> `confirm.rs:149-152` | **Yes** | Line count only; **NO file path named**. | **Medium (`P2-07` / `P1-23`):** Opens `"discard 0 selected lines?"` on context lines; `app.rs:974-980` uses bare `?` (crashes TUI). |
| 6 | **Force push branch (`+refs/heads/<branch>`)** | `src/tabs/status.rs:565-582, 874-881` (`P`) -> `confirm.rs:188-196` | **Yes** | Truncated branch; **NO remote named**. | **High (`P2-07` / `P2-08`):** `confirm.rs:194` `rsplit('/').next().expect(...)` hides `feature/` prefix; raw `--force` (`push.rs:162`); `Status::can_push` (`612-619`) blocks force-push when `ahead == 0`. |
| 7 | **Push tags** | `src/tabs/revlog.rs:482-484` (`p`), `src/popups/taglist.rs:260-264` (`p`) | **No** | N/A | **Medium (`P2-08` / `P1-23`):** Single keypress pushes all unpushed tags without remote preview; pressing `p` in `Revlog` with no remote crashes app (`app.rs:846`). |
| 8 | **Delete local branch** | `src/popups/branchlist.rs:150-154, 679-690` (`D`) -> `confirm.rs:153-161` | **Yes** | Names `branch_ref`. | **Low (`P2-07`):** No unmerged-commits warning (`branch/mod.rs:433` calls `branch.delete()`). |
| 9 | **Delete remote branch** | `src/popups/branchlist.rs:150-154, 679-690` (`D`) -> `confirm.rs:162-170` -> `app.rs:1060-1086` | **Yes** | Names `branch_ref`. | **CRITICAL (`P1-22`):** `app.rs:1066-1079` takes `rsplit('/').next()` and drops remote name -> **can delete wrong branch on wrong remote**. |
| 10 | **Delete local tag** | `src/popups/taglist.rs:228-244` (`D`) -> `confirm.rs:175-183` | **Yes** | Names `tag_name`. | **High (`P1-23`):** `app.rs:1048-1053` runs `get_default_remote()?` immediately after delete, **crashing TUI if repo has 0 or >1 non-origin remotes**. |
| 11 | **Delete remote tag** | `src/app.rs:1048-1053` -> `confirm.rs:184-187` | **Yes** | Names remote; **NO `tag_name`** (`_tag_name` ignored!). | **Medium (`P2-07`):** Prompt omits tag name, triggers even if tag isn't on remote, `PushPopup` title says `"Push"` (`push.rs:265-270`) and passes `PrePushTarget::Branch` (`push.rs:152`). |
| 12 | **Delete remote** | `src/popups/remotelist.rs:142-145, 444-451` (`r`) -> `confirm.rs:171-174` | **Yes** | Names `remote_name`. | **Low (`P2-08`):** Lowercase `r` (`key_list.rs:223`) inconsistent with `D` used for every other delete. |
| 13 | **Drop stash(es)** | `src/tabs/stashlist.rs:65-75, 207-211` (`D`) -> `confirm.rs:135-140` | **Yes** | Count + short hashes (`strings.rs:213-230`). | **Low (`P2-11`):** Multi-drop loop leaves stale UI on mid-loop failure; `event()` returns `NotConsumed`. |
| 14 | **Pop stash** | `src/tabs/stashlist.rs:77-83, 200-201` (`Enter`) -> `confirm.rs:141-144` | **Yes** | **NO stash hash/msg** (`CommitId` ignored at `:141`). | **Low (`P2-07`):** Should display stash hash/subject; `event()` returns `NotConsumed`. |
| 15 | **Abort merge** | `src/tabs/status.rs:909-918` (`A`) -> `confirm.rs:201-204` | **Yes** | `"This will revert all uncommitted changes..."` | **High (`P2-07`):** `abort_pending_state` (`merge.rs:39-50`) calls `reset_workdir(repo_path, "*")` (`remove_untracked(true)`), **deleting ALL untracked files repo-wide** without warning! |
| 16 | **Abort revert** | `src/tabs/status.rs:925-931` (`A`) -> `confirm.rs:209-212` | **Yes** | `"This will revert all uncommitted changes..."` | **High (`P2-07`):** Same `abort_pending_state` **deletes ALL untracked files repo-wide** without warning! |
| 17 | **Abort rebase** | `src/tabs/status.rs:919-924` (`A`) -> `confirm.rs:205-208` | **Yes** | `"Abort rebase?"` | **Low:** Confirm exists; could show rebase step/branch. |
| 18 | **Undo last commit (`reset --soft HEAD~`)** | `src/tabs/status.rs:596-599, 899-908` (`U`) -> `confirm.rs:213-216` | **Yes** | **NO commit hash/subject** or soft-reset note. | **Medium (`P2-08`):** `event()` ignores `!pending_rebase()` (`commands()` hides at `:774`); sets `NeedsUpdate::ALL` before confirm. |
| 19 | **Revert commit** | `src/tabs/revlog.rs:223-230, 511-522` (`D`) | **NO** | None | **High (`P2-08`):** Single keypress `D` modifies worktree/index & enters `RepoState::Revert` with **zero confirmation**. |
| 20 | **Merge / Rebase branch** | `src/popups/branchlist.rs:155-172, 348-378` (`m` / `R`) | **NO** | None | **High (`P2-08`):** Single keypress rewrites history (`rebase`) or modifies HEAD/worktree (`merge`) with **zero confirmation**. |
| 21 | **Checkout commit (detached HEAD)** | `src/components/commitlist.rs:166-176, 860-865` (`S`) | **NO** | None | **High (`P2-08`):** No detached-HEAD confirm; leaks into `StashList` (`stashlist.rs:195`), checking out raw stash commit! |
| 22 | **Update submodule** | `src/popups/submodules.rs:186-205` (`u`) | **NO** | None | **Medium (`P2-08`):** Single keypress runs synchronous `submodule.update(true, ...)` (`submodules.rs:104`), blocking UI & updating submodule HEAD without confirm. |
| 23 | **Pull non-FF merge / rebase** | `src/popups/pull.rs:155-158, 188-199` -> `confirm.rs:197-200` | **Yes** | Incoming count + mode; **NO branch/remote**. | **Low:** Could name current branch & upstream. |
| 24 | **Amend / Reword commit** | `src/popups/commit.rs:322-338` (`Ctrl+A`), `revlog.rs:558-571` (`r`) | **Implicit** | Editor title only; no pushed-history warning. | **Low:** `reword` rebases descendants (`sync::reword`) without warning if upstream exists. |

### 4.2 Modal Stacking, Esc-Cancel Bugs & Key-Hint Mismatches (`STEP 3C`)

- **Modal Z-Order vs Event-Capture Inversion (`P2-09`, `src/app.rs:316-368, 495-572`):** `accessors!` and `setup_popups!` are not reverses (`log_search_popup`/`fuzzy_find_popup` capture events before `msg_popup`/`confirm_popup` while drawing underneath them; `goto_line_popup` draws over `msg_popup` while `msg_popup` steals events; `help_popup` draws beneath 28 popups; `App::event` doesn't guard `tab_toggle`/`switch_tab`/`open_options` behind `!self.any_popup_visible()`).
- **Esc-Cancel Executes Network Ops + Modal Lockup (`P2-10`):**
  - `push.rs:315-326`, `push_tags.rs:237-247`, `pull.rs:262-273`, `fetch.rs:175-186`: Condition `if self.input_cred.get_cred().is_complete() || !self.input_cred.is_visible()` treats `Esc` cancel (`hide()` makes `!is_visible()` true) as completion and fires the push/pull/fetch with empty creds!
  - `pull.rs:70-87, 249-278` & `fetch.rs:62-80, 162-191` & `app.rs:829-844`: Call `self.show()?` *before* preflight errors, leaving an unclosable popup (`visible = true, pending = false`) with no `exit_popup`/`enter` handler — permanent modal trap!
- **Modal Event Leaks (`P2-11`):** `RevisionFilesPopup` (`revision_files.rs:123-145`) leaks `Esc` and unhandled keys; `TagListPopup` (`taglist.rs:228-260`) leaks keys on empty tag list; `InspectCommitPopup` (`inspect_commit.rs:174-192`) leaks `open_file_tree` when `open_request.is_none()`; `StashList::event` (`stashlist.rs:199-221`) falls through to `NotConsumed`; `TabSwitchStatus` (`app.rs:803`) skips `NeedsUpdate::ALL | COMMANDS`; `FuzzyFindPopup` (`fuzzy_find.rs:110-129, 336-375`) keeps filter selection on `Esc`, double-processes `Up`/`Down`, and forgets `find_text.hide()`.
- **Key-Hint Mismatches (`P2-12`):**
  - `MsgPopup` (`msg.rs:121-148`): advertises `move_up`/`move_down` (`navigate_commit_message`) while listening to `popup_up`/`popup_down`.
  - `PushPopup` (`push.rs:302-306, 327-333`), `PushTagsPopup` (`push_tags.rs:225-229, 248-254`), `PullPopup` (`pull.rs:249-253`), `FetchPopup` (`fetch.rs:162-166`): show `Close [Enter]` (`close_msg`) while handling only `Esc` (or neither!).
  - `CheckoutOptionPopup` (`checkout_option.rs:173-189`): copy-pasted `"confirm reset"` / `"change reset type"`.
  - `InspectCommitPopup` (`inspect_commit.rs:111-130, 171`): emits `close_popup` twice and binds `move_left` to close without hint.
  - `CompareCommitsPopup` (`compare_commits.rs:98-137`) & `FileRevlogPopup` (`file_revlog.rs:494-627`): show broken `Back [←]` (`DiffComponent::event` `diff.rs:871-875` always eats `move_left`), `CompareCommits` closes on `move_left`, and `FileRevlog` omits `self.diff.commands()` & `scroll`.
  - `CreateRemotePopup` (`create_remote.rs:62-68`): shows `"Confirm name [Enter]"` during `State::Url`.
  - `RemoteListPopup` (`remotelist.rs:115-119`): hides `Add [a]` when there are 0 remotes!
  - `BranchListPopup` (`branchlist.rs:136-196, 707-788`): `event()` ignores `!self.local` on rename, `has_remotes` checks branches instead of remotes, `compare`/`enter` enabled/disabled mismatch, and `reset_branch` says `"confirm reset"` (`strings.rs:1628`).
  - `TagCommitPopup` (`tag_commit.rs:52-65, 84-96`): `tag_annotate` (`Ctrl+A`) overwrites `tag_name` when already in `Mode::Annotation`; `enter` skips `is_valid_tag()`.
  - `LogSearchPopupPopup` (`log_search.rs:508-596`): `JumpCommitSha` `Esc` goes back rather than closing while showing `Close [Esc]`; hides search hints under `force_all`.
  - `OptionsPopup` (`options.rs:302-311`): uses `"navigate tree view, collapse, expand"`.
  - `Revlog` (`strings.rs:1592`): `log_find_commit` shows `file_find` instead of `log_find`.
  - Most popups swallow `cmd_bar_toggle` (`.`), breaking `"more [.]"`.

### 4.3 Silent Failures (`P2-21`) & Text Input Validation Matrix (`P2-23` -> `STEP 3C`)

- **Silent Async Failures (`P2-21`):**
  - `FetchPopup` (`fetch.rs:105-114`) + `AsyncFetchJob` (`asyncgit/src/fetch_job.rs:13-68`): drops `JobState::Response(Err(_))` with no getter or `ShowErrorMsg` -> requires `STEP 3A` getter + `STEP 3C` error popup.
  - `BlameFilePopup` (`blame_file.rs:345-353, 441-480`) + `AsyncBlame` (`asyncgit/src/blame.rs:117-123`): blame error logs only and leaves `BlameProcess::GettingBlame` spinning forever; also lacks `hide()`.
  - `TagListPopup` (`taglist.rs:348-353`): swallows `AsyncRemoteTagsJob` errors.
  - `RemoteListPopup::update_remote_url` (`remotelist.rs:460-471`): silently no-ops when `get_remote_url` returns `Ok(None)` or `Err(_)`.
- **Validation & Error-State Matrix (`P2-23`, `STEP 3C`):**

| Popup (`file:line`) | Validation Gap | Error / Reopen State Defect |
|---|---|---|
| `CreateBranchPopup` (`create_branch.rs:47-53,83-85,134-156`) | Allows Enter on empty or `[invalid name]` | Clears & hides on error; `Esc` leaves stale text |
| `RenameBranchPopup` (`rename_branch.rs:47-53,83-85,142-174`) | Allows Enter on empty or `[invalid name]` | `line 173` wipes `input` underneath error popup! |
| `TagCommitPopup` (`tag_commit.rs:151-153,164-206`) | Allows whitespace/spaces/invalid git ref chars; empty annotation allowed on Enter | `open()` keeps stale text; hides & discards annotation on error |
| `CreateRemotePopup` (`create_remote.rs:170-213`) | Allows empty/invalid remote name & empty/whitespace URL | Hides on error, losing entered name & URL |
| `RenameRemotePopup` (`rename_remote.rs:118-175`) | Allows empty/invalid remote name; uses `branch_name_invalid` | Clears & hides on error |
| `UpdateRemoteUrlPopup` (`update_remote_url.rs:127-151`) | No empty/whitespace URL check; `initial_url` unused | Clears & hides on error; never resets `remote_name` |
| `CommitPopup` (`commit.rs:209-278,303-320,340-346,446-452`) | Allows whitespace/comment-only msg; signoff silent fail; `prepare-commit-msg` error swallowed | Resets `self.verify = true` even when commit fails/aborts |
| `StashMsgPopup` (`stashmsg.rs:60-98`) | Allows whitespace-only message `Some("   ")` | Prints raw `{:?}` `StashingOptions` in error popup; stale text on Esc/error |
| `GotoLinePopup` (`goto_line.rs:92-141`) | — | Stale `invalid_input` triggers error on empty input after backspace or reopen |
| `SubmodulesListPopup` (`submodules.rs:424`) | — | Renders raw `{:?}` `SubmoduleStatus(...)` debug flags |

---

## Section 5: Styling & Design-Token Audit — Nebulous & Orrery Mapping (Auditor 5 + Direct Extraction)

### 5.1 Canonical Nebulous & Orrery Design System (Verified Direct Source)

Both the Nebulous repo (`crates/nebulous_cli/src/tui/theme.rs:6-107`, `render.rs:20-583`) and the Orrery repo (`crates/orrery-tui/src/theme.rs:5-82`, `widgets.rs:48-662`) share an **exact byte-for-byte 7-color P3 Phosphor Amber-on-Black CRT design system**:

| Token Name | RGB / Hex | Verified Source Citations | Semantic Role |
|---|---|---|---|
| `BG_BLACK` | `Color::Rgb(0, 0, 0)` (`#000000`) | `Nebulous/theme.rs:6`, `Orrery/theme.rs:5` | Pure CRT canvas background (`paint_crt_background` / base frame) |
| `PANEL_BG` | `Color::Rgb(12, 8, 0)` (`#0C0800`) | `Nebulous/theme.rs:9`, `Orrery/theme.rs:6` | Elevated warm dark phosphor surface for focused panels, selection rows, and modals |
| `AMBER_PRIMARY` | `Color::Rgb(255, 176, 0)` (`#FFB000`) | `Nebulous/theme.rs:12`, `Orrery/theme.rs:7` | Primary foreground text, active/focused borders (`+ BOLD` in Nebulous), command labels, passed/success states |
| `AMBER_BRIGHT` | `Color::Rgb(255, 216, 102)` (`#FFD866`) | `Nebulous/theme.rs:15`, `Orrery/theme.rs:8` | Focused titles (`+ BOLD`), active tabs (`+ BOLD`), selected row fg (`+ BOLD`), footer/cmdbar `[key]` badges (`+ BOLD`) |
| `AMBER_DIM` | `Color::Rgb(153, 99, 0)` (`#996300`) | `Nebulous/theme.rs:18`, `Orrery/theme.rs:9` | Unfocused pane borders, inactive tabs, secondary/muted labels, disabled actions |
| `AMBER_DARK` | `Color::Rgb(92, 58, 0)` (`#5C3A00`) | `Nebulous/theme.rs:21`, `Orrery/theme.rs:10` | Subtle structural dividers (`" │ "`, `"─"`, `"═"`), scrollbar tracks, low-emphasis grid separators |
| `AMBER_ALERT` | `Color::Rgb(255, 85, 0)` (`#FF5500`) | `Nebulous/theme.rs:25`, `Orrery/theme.rs:11` | Danger/error/alert borders, high/critical severity, failure states (`+ BOLD`) |

**Canonical Chrome Rules Observed in Reference Repos:**
1. **Border Set:** `Block::default().borders(Borders::ALL)` — **implicit `BorderType::Plain`** (`┌─┐│└─┘`) across every panel and modal in both Nebulous (`render.rs:154-156, 269-279, 579-582`) and Orrery (`theme.rs:52-71`). *Note:* Does NOT use `BorderType::Rounded` (corrects the placeholder example in `STEPPED_IMPLEMENTATION_PLAN.md:352`) and keeps `src/gitui.rs:229` (`│`) snapshot filter working!
2. **Title Formatting:** Single-space padded `" {title} "` (`Nebulous/render.rs:271, 328, 575`; `Orrery/widgets.rs:48, 639, 664`). Focused title uses `AMBER_BRIGHT + BOLD`; unfocused uses `AMBER_PRIMARY` or `AMBER_DIM`.
3. **Focus Indicators:**
   - Focused pane: `border_style` = `AMBER_PRIMARY` (+ `BOLD` in Nebulous; `PANEL_BG` fill in Orrery `focused_block()`).
   - Unfocused pane: `border_style` = `AMBER_DIM` on `BG_BLACK`.
   - Alert/Attention pane: `border_style` = `AMBER_ALERT` (`Orrery/theme.rs:66-71`, `Nebulous/render.rs:96-103`).
4. **Selection Highlight:** `fg(AMBER_BRIGHT).bg(PANEL_BG).add_modifier(Modifier::BOLD)` (`Nebulous/theme.rs:90-95`) or `highlight().bg(bg)` (`Orrery/theme.rs:77-82`, `widgets.rs:326`), prefixed with `▸ ` (`Nebulous/render.rs:23-24`) when list-based.
5. **Tab / Stage Bar:** Active tab in `AMBER_BRIGHT + BOLD` (Nebulous wraps `▶ [n] Title ◀` at `render.rs:179-182`; Orrery uses `highlight()`), inactive tabs in `AMBER_DIM`, separators `" │ "` and bottom rule `─` in `AMBER_DARK` (`Nebulous/render.rs:175-206`).
6. **Footer / Command Bar:** Key hints `[key]` in `AMBER_BRIGHT + BOLD` (`Nebulous/render.rs:531-553`), action label in `AMBER_PRIMARY` (`bg(BG_BLACK)` or `bg(PANEL_BG)`), status/alerts prefixed with `AMBER_ALERT` or `AMBER_BRIGHT`.
7. **Modal Overlay:** `Clear` + `PANEL_BG` background paint + `Block::default().borders(Borders::ALL)` (`BorderType::Plain`) with `AMBER_PRIMARY` border (`AMBER_ALERT` for errors/confirmations) and `AMBER_BRIGHT + BOLD` title (`Nebulous/render.rs:567-583`, `Orrery/widgets.rs:629-661`).

### 5.2 Backwards Compatibility (`ThemePatch` & `~/.config/gitui/theme.ron`)

- `src/ui/style.rs:12-39` derives `struct_patch::Patch` over **24 fields** (`selected_tab`, `command_fg`, `selection_bg`, `selection_fg`, `use_selection_fg`, `cmdbar_bg`, `disabled_fg`, `diff_line_add`, `diff_line_delete`, `diff_file_added`, `diff_file_removed`, `diff_file_moved`, `diff_file_modified`, `commit_hash`, `commit_time`, `commit_author`, `danger_fg`, `push_gauge_bg`, `push_gauge_fg`, `tag_fg`, `branch_fg`, `line_break`, `block_title_focused`, `syntax`).
- **Constraints:**
  1. Never rename or remove any of the 24 existing fields.
  2. Keep `line_break = "¶"` (`src/components/diff.rs:975-1035` asserts `"¶"`).
  3. Keep `selection_bg != Color::Black` and `syntax != "InspiredGitHub"` (`src/ui/style.rs:395-396` `test_smoke` asserts inequality).
  4. **Hazard (`src/ui/style.rs:286-290, 315-324`):** `load_old_theme` deserializes into full `Theme` without `#[serde(default)]`. **Add `#[serde(default)]` to `pub struct Theme`** before adding any optional field so legacy `< 0.23` `theme.ron` files keep migrating cleanly.
  5. **Third-party `theme.ron` compatibility (`THEMES.md:39`):** Map new chrome semantics (`block(true)` -> `block_title_focused`, `attention_block_style(&self)` -> `diff_file_modified`, `scroll_bar_track(&self)` -> `disabled_fg`, `commandbar_key` -> `selected_tab + BOLD`) onto existing fields wherever semantically aligned so user themes (e.g. Catppuccin) do not get mismatched Amber borders.
  6. **Contrast bug (`src/ui/style.rs:105-107`):** `Theme::text(true, true)` uses `command_fg` instead of `selection_fg` (`apply_select`). Ensure `selection_fg` and `command_fg` both contrast cleanly against `selection_bg` (or align `text(true, true)` with `apply_select`).

### 5.3 P3 Design Token Mapping Table (`STEP 4A` / `STEP 4B` / `STEP 4C`)

| Semantic Role | Nebulous / Orrery Token | `Theme` Field (`src/ui/style.rs`) | Proposed Helper / Default Value | Step & Target Files |
|---|---|---|---|---|
| **Terminal / Canvas BG** | `BG_BLACK` (`#000000`) | *(new optional `canvas_bg` or `Color::Reset`)* | Paint `BG_BLACK` / keep `Color::Reset` per user decision | `STEP 4A/4B`: `style.rs`, `app.rs` |
| **Elevated Panel / Modal BG** | `PANEL_BG` (`#0C0800`) | *(new optional `panel_bg` or `PANEL_BG`)* | `popup_surface(&self)` / `PANEL_BG` | `STEP 4A/4B`: `style.rs`, `ui/mod.rs`, `popups/*` |
| **Primary Text FG** | `AMBER_PRIMARY` (`#FFB000`) | `command_fg` (`:16`) | `Color::Rgb(255, 176, 0)` | `STEP 4A`: `style.rs` |
| **Muted / Disabled FG** | `AMBER_DIM` (`#996300`) | `disabled_fg` (`:21`) | `Color::Rgb(153, 99, 0)` | `STEP 4A`: `style.rs` |
| **Focused Pane Border** | `AMBER_PRIMARY` (`#FFB000`) | `block_title_focused` (`:37`) | `block(&self, true)` -> `.fg(self.block_title_focused)` *(currently `Style::default()`!)* | `STEP 4A/4B`: `style.rs`, `ui/mod.rs` |
| **Unfocused Pane Border** | `AMBER_DIM` (`#996300`) | `disabled_fg` (`:21`) | `block(&self, false)` -> `.fg(self.disabled_fg)` | `STEP 4A/4B`: `style.rs`, `ui/mod.rs` |
| **Focused Pane Title** | `AMBER_BRIGHT + BOLD` (`#FFD866`) | `block_title_focused` (`:37`) / `selected_tab` | `title(&self, true)` + `" {title} "` padding | `STEP 4A/4B`: `style.rs`, `ui/mod.rs` |
| **Unfocused Pane Title** | `AMBER_DIM` (`#996300`) | `disabled_fg` (`:21`) | `title(&self, false)` + `" {title} "` padding | `STEP 4A/4B`: `style.rs`, `ui/mod.rs` |
| **Border Set** | `BorderType::Plain` (`┌─┐│└─┘`) | *(layout chrome)* | `ui::styled_block` & `ui::popup_block` (replaces mixed `Plain`/`Thick`) | `STEP 4B`: `ui/mod.rs`, `scrolllist.rs`, `components/*`, `popups/*` |
| **Active Tab** | `AMBER_BRIGHT + BOLD` (`#FFD866`) | `selected_tab` (`:15`) | `Color::Rgb(255, 216, 102)` + `BOLD` (+ `UNDERLINED` or `▶ … ◀`) | `STEP 4A/4B`: `style.rs`, `app.rs`, `branchlist.rs` |
| **Inactive Tab & Dividers** | `AMBER_DIM` / `AMBER_DARK` (`#5C3A00`) | `disabled_fg` (`:21`) | `tab(false)`, `tab_divider(&self)` (`│`, `─`) | `STEP 4A/4B`: `style.rs`, `app.rs` |
| **Selection BG / FG** | `PANEL_BG` (`#0C0800`) *(or `#2A1A00`)* / `AMBER_BRIGHT` | `selection_bg` (`:17`), `selection_fg` (`:18`), `use_selection_fg` (`:19`) | `selection_bg = Rgb(12, 8, 0)` *(verify contrast vs `#2A1A00`)*, `selection_fg = Rgb(255, 216, 102)`, `use_selection_fg = true` | `STEP 4A`: `style.rs` |
| **Command Bar Surface BG** | `BG_BLACK` / `PANEL_BG` | `cmdbar_bg` (`:20`) | `Color::Rgb(12, 8, 0)` + `commandbar_bg(&self)` filling full bar | `STEP 4A/4B`: `style.rs`, `cmdbar.rs` |
| **Command Bar Key Badge** | `AMBER_BRIGHT + BOLD` (`#FFD866`) | `selected_tab` (`:15`) | New `commandbar_key(&self, enabled)` splitting `"Label [key]"` | `STEP 4A/4B`: `style.rs`, `cmdbar.rs` |
| **Command Bar Label / Splitter** | `AMBER_PRIMARY` / `AMBER_DARK` | `command_fg` / `disabled_fg` | `commandbar(&self, enabled)`, `commandbar_splitter(&self)` | `STEP 4A/4B`: `style.rs`, `cmdbar.rs` |
| **Semantic: Danger / Delete** | `AMBER_ALERT` (`#FF5500`) | `danger_fg`, `diff_line_delete`, `diff_file_removed` | `Color::Rgb(255, 85, 0)` | `STEP 4A/4B`: `style.rs`, `status.rs`, `goto_line.rs` |
| **Semantic: Add / OK / Author** | `AMBER_BRIGHT` *(or `#7FD962`)* / `AMBER_PRIMARY` | `diff_line_add`, `diff_file_added`, `commit_author` | Confirm monochrome vs warm phosphor green (`#7FD962`) | `STEP 4A`: `style.rs` |
| **Semantic: Warn / Modify / Branch** | `AMBER_PRIMARY` / `AMBER_BRIGHT` / `AMBER_ALERT` | `diff_file_modified`, `branch_fg` | Replace static `attention_block()` with `attention_block_style(&self)` | `STEP 4A/4B`: `style.rs`, `status.rs`, `revlog.rs` |
| **Semantic: Info / Time / Hash / Tag** | `AMBER_PRIMARY` / `AMBER_DIM` / `AMBER_BRIGHT` | `commit_hash`, `commit_time`, `tag_fg`, `push_gauge_bg/fg` | Distinct amber hierarchy (`#FFD866`, `#FFB000`, `#996300`) + `info_title` | `STEP 4A/4B`: `style.rs`, `msg.rs` |
| **Scrollbar & Syntax** | `AMBER_PRIMARY` / `AMBER_DARK` | `selection_bg` / `disabled_fg`, `syntax` | `scroll_bar_pos(&self)`, `scroll_bar_track(&self)`, `base16-eighties.dark` | `STEP 4A/4B`: `style.rs`, `scrollbar.rs` |

### 5.4 Hardcoded Styles, Broken `Block` Sites & Top-Down Container Tree (`STEP 4B` / `4C`)

- **14 Hardcoded Style Sites Bypassing `Theme`:**
  - `src/ui/style.rs:276-278` (static `attention_block()` hardcodes `Color::Yellow`), `src/tabs/status.rs:309` (`Color::Red`), `src/popups/goto_line.rs:152` (`Color::Red`), `src/components/textinput.rs:171-177, 208-209` (unstyled bold border, unstyled title & char count), `src/cmdbar.rs:143-145, 174-177, 188-195` (unstyled splitter/bar/`more [.]`), `src/ui/scrollbar.rs:34-35, 139-141` (`Style::default()` track & heavy `DOUBLE_VERTICAL`), `src/spinner.rs:51-53` (unstyled backend cell), `src/popups/help.rs:85, 224` (`Style::default()` & `REVERSED`), `src/popups/options.rs:111-112` (`UNDERLINED` TODO), `src/tabs/status.rs:222-225` (unstyled branch badge), `src/popups/commit.rs:102-103` (unstyled branch badge), `src/tabs/stashing.rs:102-129` (`Span::raw`), `src/components/commitlist.rs:468` (`Style::default()`).
- **Broken & Mixed `Block` Sites (29 sites across 24 files):**
  - Swapped/buggy styles: `src/components/syntax_text.rs:210-214` (passes `theme.title(...)` to `.border_style` & leaves title unstyled!), `src/popups/msg.rs:91-98, 203-208` (`show_info` gets red danger title & no `.border_style`), `src/popups/fuzzy_find.rs:277-284` & `src/popups/log_search.rs:364-370, 420-427` (`.style(theme.title(true))` leaks bold onto content), `src/popups/externaleditor.rs:139-145` (`.style(theme.block(true))` colors body text with border color!).
  - Completely unstyled `Block`s: `goto_line.rs:158`, `stashing.rs:157-159`, `help.rs:46-49`, `remotelist.rs:54-57, 315-318`, `submodules.rs:63-66, 487, 512`, `branchlist.rs:77-80`.
  - Hardcoded focus state: `commitlist.rs:805-812` (always `true`), `commit_details/details.rs:275` (always `false`), `commit_details/compare_details.rs:121, 134` (always `false`).
  - Conflicting `BorderType`: 17 `Plain` sites vs 12 `Thick` popup sites (`popups/mod.rs:91`, `externaleditor.rs:141`, `fetch.rs:134`, `msg.rs:97`, `help.rs:49`, `remotelist.rs:56`, `push_tags.rs:196`, `taglist.rs:106`, `push.rs:274`, `pull.rs:221`, `submodules.rs:65`, `branchlist.rs:79`).
- **Top-Down Container Tree (for `STEP 4B` `[PRE_GATE]`):**
  - Root `Layout::Vertical` (`src/app.rs:263-273`):
    1. `Length(2)` Top Tab Bar (`src/app.rs:1160-1228`): `(0,0)` `Spinner` -> `Tabs` + right-aligned `repo_path` -> bottom `Borders::BOTTOM` rule.
    2. `Min(2)` Active Workspace Tab (`Status`, `Revlog`, `FilesTab`, `Stashing`, `StashList`) with focused/unfocused `ui::styled_block`.
    3. `Length(cmdbar.height())` Command Bar (`src/cmdbar.rs:139-199`) with full-width surface fill, key badge split, and dynamic `more/less [<key>]`.
    4. Modal Overlay (`Clear` + `PANEL_BG` + `ui::popup_block`, above cmdbar).
- **Affected `insta` Snapshots (`STEP 4C`):**
  - `src/snapshots/gitui__gitui__tests__app_loading.snap`
  - `src/snapshots/gitui__gitui__tests__app_loading_finished.snap`
  - `src/snapshots/gitui__gitui__tests__app_log_tab_showing_one_commit.snap`
  - *Note:* `STEP 4A` (color-only) leaves snapshots unchanged (`TestBackend` compares text); `STEP 4B` (padded titles `" {title} "`, cmdbar/scrollbar/divider tweaks) updates all 3 snapshots. Keeping `BorderType::Plain` (`│`) avoids breaking `src/gitui.rs:229`.

---

## Section 6: Master P0–P3 Remediation Table (Mapped to Steps 2A–4C)

| ID | Priority | Target Step | Primary File(s):Line | Finding Summary | Concrete Remediation Spec | Scope-Lock Note |
|---|---|---|---|---|---|---|
| `P0-01` | P0 | STEP 2A | `.github/workflows/ci.yml:3-5` | Daily upstream cron schedule | Remove `schedule:` trigger; keep `push`/`pull_request` on `master` | In scope |
| `P0-02` | P0 | STEP 2A | `.github/workflows/ci.yml:15-100` | Multi-OS/nightly build matrix & OS steps | Restrict `build` to `ubuntu-latest` (`[stable, "1.88"]`), remove macOS/Windows steps | In scope |
| `P0-03` | P0 | STEP 2A | `.github/workflows/ci.yml:68-70` | `make clippy` skips tests, omits `-D warnings`, collides `regex-*` | Run `cargo clippy --workspace --all-targets -- -D warnings` directly | In scope |
| `P0-04` | P0 | STEP 2A | `.github/workflows/ci.yml:101-157` | `build-linux-musl` matrix, `apt-get`, `rustup target` order, `cargo install` | Scope to `stable`, run `apt-get update && apt-get install -y musl-tools gpgsm`, fix target setup & drop non-target install | In scope |
| `P0-05` | P0 | STEP 2A | `.github/workflows/ci.yml:158-355` | 5 upstream jobs (`arm`, `apple`, `udeps`, `log-test`, `test-homebrew`) | Delete all 5 upstream-only jobs | In scope |
| `P0-06` | P0 | STEP 2A | `.github/workflows/ci.yml:284` | `cargo fmt -- --check` misses workspace crates | Change to `cargo fmt --all -- --check` | In scope |
| `P0-07` | P0 | STEP 2A | `.github/workflows/nightly.yml:1-125` | Upstream S3 nightly cron missing `gpgsm`/secrets | Delete `.github/workflows/nightly.yml` | In scope |
| `P0-08` | P0 | STEP 2A | `.github/workflows/brew.yml:1-23` | Upstream Homebrew formula workflow | Delete `.github/workflows/brew.yml` | In scope |
| `P0-09` | P0 | STEP 2A | `.github/workflows/cd.yml:1-134` | Upstream multi-OS/Homebrew tag release workflow | Trim to Linux-only `ubuntu-latest` (with `gpgsm`/`musl-tools`) or disable | In scope |
| `P1-01` | P1 | STEP 2A | `.github/workflows/ci.yml:297-299` | Unpinned `cargo install cargo-deny` from source | Switch to pinned prebuilt `taiki-e/install-action@cargo-deny` | In scope |
| `P2-01` | P2 | STEP 2A / 2B | `.github/workflows/ci.yml:25-277`, `.github/dependabot.yml:6`, `Cargo.toml:7,17-24` | Cache key drift, `rustup override` vs `rust-toolchain.toml`, member `rust-version`, `invalidstring` workspace member | Standardize `shared-key`, verify checkout action, optionally set `rust-version.workspace = true` | In scope |
| `P0-10` | P0 | STEP 2B | `Cargo.lock:2470` | `libz-sys 1.1.21` (`RUSTSEC-2026-0418` + `RUSTSEC-2026-0544`) | Update `libz-sys` to `>= 1.1.33` cleanly in `Cargo.lock` | In scope |
| `P0-11` | P0 | STEP 2B | `Cargo.toml:35,76`, `Cargo.lock:343` | `chrono 0.4.44` (`RUSTSEC-2026-0543` unmaintained) | Migrate 5 files to `jiff 0.2.32` OR get user sign-off on `deny.toml` ignore | **User decision; if migrating, add 5 files** |
| `P0-12` | P0 | STEP 2B | `Cargo.lock:2530,862,3284,3056,2015` | `lru`, `faster-hex`, `rustix`, `rand` advisories (+ check `heapless`) | Update `ratatui-core >= 0.1.2`, `faster-hex >= 0.10.1`, `rustix >= 1.1.6`, `rand >= 0.8.6`; verify `heapless` post-fetch | In scope |
| `P0-13` | P0 | STEP 2B | `asyncgit/Cargo.toml:41`, `Cargo.lock:3318,2942` | `scc 2.3.4` (`RUSTSEC-2026-0205`) & `pin-utils` (`RUSTSEC-2026-0441`) via `serial_test` | Disable `serial_test` default features (`futures-util`/`pin-utils`) & update `scc >= 3.8.4` | In scope |
| `P0-14` | P0 | STEP 2B | `deny.toml:15,26-62` | Legacy `version = 2` / `{ name = }` syntax & 3 stale `skip-tree` entries | Migrate to `cargo-deny 0.20` `{ crate = }`, prune stale entries, verify `bans`/`licenses` post-fetch | In scope |
| `P0-15` | P0 | STEP 2B | `.clippy.toml:1-2`, `asyncgit/src/revlog.rs:370,403` | >1,270 `#[cfg(test)]` unwrap/expect/panic errors under `--all-targets` + `unit_cmp` | Enable `allow-{unwrap,expect,panic}-in-tests = true` in `.clippy.toml`; fix all Clippy diagnostics | **Add `.clippy.toml` & `Makefile` to STEP 2B** |
| `P2-02` | P2 | STEP 2B | `deny.toml`, `build.rs:8-47` | Missing `[sources]`/`[graph]`, `build.rs` slice/unwrap & unconditional warnings | Add `[sources]` + Linux `[graph] targets` to `deny.toml`; guard `build.rs` slice/parse if in scope | Optional `build.rs` |
| `P0-16` | P0 | STEP 2C | `Makefile:2,21-110` | `Makefile` hard-requires `cargo-nextest`, misses `--all` on `fmt` & `--all-targets` on `clippy` | Align `Makefile` targets with Linux CI & `cargo test --workspace` parity | **Add `Makefile` to STEP 2B/2C** |
| `P0-17` | P0 | STEP 2C | `asyncgit/src/revlog.rs:391-401`, `asyncgit/src/sync/sign.rs:755,870,907,991` | Process-wide `GIT_DIR`/`GNUPGHOME` env races/leaks under `cargo test` | Wrap env mutations in RAII `Drop` guards and fix any test failures post-fetch | In scope |
| `P1-02` | P1 | STEP 3A | `asyncgit/src/sync/cred.rs:100-175` | Repo `.git/config` `credential.helper` RCE & `credential.username` newline injection | Use `Config::open_default()` only; validate/strip URL userinfo via `url::Url` | In scope |
| `P1-03` | P1 | STEP 3A | `asyncgit/src/sync/cred.rs:50-159`, `remotes/{callbacks.rs:179-220,mod.rs:302-334,push.rs,tags.rs}`, `hooks.rs` | `pushurl` mismatch, loose `starts_with("http")`, `fetch_all` cred broadcast, unbound `Callbacks` | Check `pushurl`, parse HTTP/HTTPS scheme, bind `Callbacks::with_expected_url`, per-remote creds in `fetch_all` | **Add `remotes/*` to STEP 3A** |
| `P1-04` | P1 | STEP 3A | `asyncgit/src/sync/cred.rs:16-22`, `remotes/callbacks.rs:185-187` | `#[derive(Debug)]` & raw URL logging leak passwords/tokens | Manual `Debug` redaction + `Drop` zeroing on `BasicAuthCredential`; redact URL userinfo in `callbacks.rs` | **Add `remotes/callbacks.rs`** |
| `P1-05` | P1 | STEP 3A | `git2-hooks/src/hookspath.rs:42-133,296-310` | `shellexpand::full` `$VAR` expansion, hook traversal, dir/world-writable exec | Use `shellexpand::tilde`, validate `Component::Normal` hook, canonicalize relative `hooksPath`, require file & `!world_writable` | In scope |
| `P1-06` | P1 | STEP 3A | `git2-hooks/src/hookspath.rs:208-256` | Unix `sh` missing `--` option terminator; brittle Windows `sh -c` | Pass `--` on Unix (`sh -- <hook>`) and positional `"$0" "$@"` on Windows | In scope |
| `P1-07` | P1 | STEP 3A | `git2-hooks/src/lib.rs:195-204,297-330` | `COMMIT_EDITMSG` symlink follow, `0o644`, no cleanup, lossy path, unbounded read | Non-symlink check + atomic `create_new` + `0o600` + RAII cleanup + bounded read + `OsStr` hook args | In scope |
| `P1-08` | P1 | STEP 3A | `git2-hooks/src/lib.rs:150-176` | 4 prod `.unwrap()` & `chmod +x` subprocess in `create_hook` | Use `Permissions::from_mode(0o700)` & `Result` (`try_create_hook` if keeping semver) | In scope |
| `P1-09` | P1 | STEP 3A | `asyncgit/src/sync/sign.rs:152-362` | Relative/PATH `gpg.*program`, non-`ssh-` literal key in argv/trace, missing `--`, lossy temp path | Reject relative-with-sep/`..` program, sanitize `PATH`, temp-file all SSH keys (`0o600`), validate `signingKey`, redact trace | In scope |
| `P1-10` | P1 | STEP 3A | `asyncgit/src/sync/hooks.rs:101-106,226-278`, `git2-hooks/src/lib.rs:90-108` | `pre-push` stdin refspec newline/whitespace injection | Validate `local_ref`/`remote_ref` via `Reference::is_valid_name` & reject whitespace/controls | In scope |
| `P1-11` | P1 | STEP 3A | `asyncgit/src/sync/utils.rs:182-194` | `repo_write_file` path traversal & symlink truncation | Reject absolute/`..`, canonicalize & assert `starts_with`, refuse symlinks, atomic write-rename | **Add `sync/utils.rs` to STEP 3A** |
| `P1-12` | P1 | STEP 3A | `asyncgit/src/sync/config.rs:111-140`, `asyncgit/src/sync/mod.rs` | No global-only config reader (`Config::open_default`) | Add `get_global_config_string` for `externaleditor.rs` & `commit.rs` | **Add `sync/config.rs` & `mod.rs` to STEP 3A** |
| `P1-13` | P1 | STEP 3B | `src/popups/externaleditor.rs:69-78,112-116` | Untrusted local `core.editor` RCE & CWD binary hijack | Use `get_global_config_string("core.editor")`, reject relative sep/`..`, resolve sanitized `PATH` | In scope |
| `P1-14` | P1 | STEP 3B | `src/popups/externaleditor.rs:48-62,107-116` | External editor path traversal, symlink escape & missing `--` | Canonicalize target inside `work_dir` or `git_dir`, check `is_file()`, pass `--` before path | In scope |
| `P1-15` | P1 | STEP 3B | `src/popups/externaleditor.rs:80-108` | Broken quote/whitespace splitting for editor commands | Implement `parse_editor_command` tokenizer handling quotes/escapes cleanly + unit tests | In scope |
| `P1-16` | P1 | STEP 3B | `src/popups/externaleditor.rs:64-67` | `.expect("reset terminal")` inside `defer!` guard | Log error instead of panicking inside drop guard | In scope |
| `P1-17` | P1 | STEP 3B | `src/clipboard.rs:13-17,119` | `PATH` hijack & bare-command fallback execution | Resolve via `which_in` on absolute-only `PATH`, fail closed if absent, check non-empty `WAYLAND_DISPLAY` | In scope |
| `P1-18` | P1 | STEP 3B | `src/args.rs:71-82`, `src/ui/style.rs:293-324` | `-t`/`--theme` path traversal & arbitrary read/overwrite | Require single `Component::Normal` filename before joining to `confpath` + unit tests | In scope |
| `P1-19` | P1 | STEP 3B | `src/args.rs:195-227` | `setup_logging` creates `0o644` log & follows symlinks | Reject symlinks, open with `mode(0o600)`, enforce `0o700` on new cache dir | In scope |
| `P1-20` | P1 | STEP 3B | `src/components/cred.rs:63-161` | `CredComponent::hide()` retains password & causes blind input hijack | Hide & `.clear()` both `input_username` & `input_password` on hide/submit | **Add `src/components/cred.rs` to STEP 3B** |
| `P1-21` | P1 | STEP 3B | `src/popups/commit.rs:164-200,400-424` | `COMMIT_EDITMSG` symlink/`0o644` & `commit.template` `shellexpand::full()` RCE/read | Atomic `0o600` non-symlink edit file; `shellexpand::tilde` / global-only bounded read for template | **Add `src/popups/commit.rs` to STEP 3B** |
| `P2-03` | P2 | STEP 3B | `src/clipboard.rs:74-84,92,114` | Unbounded & unflushed OSC 52 payload | Cap payload (~75 KiB) & call `out.flush()?` | In scope |
| `P2-04` | P2 | STEP 3B | `src/args.rs:52-69,177-192` | Unvalidated `-d`/`-w` CLI/env paths | Reject empty/NUL `-d`/`-w`, verify & canonicalize `-w` directory | In scope |
| `P2-05` | P2 | STEP 3B | `src/popups/externaleditor.rs:69-118` | Ignores non-zero editor exit (`:cq`) & empty `GIT_EDITOR=""` | Skip empty env vars & abort cleanly on non-zero `ExitStatus` | In scope |
| `P1-22` | P1 | STEP 3C | `src/app.rs:1060-1086` | `delete_remote_branch` truncates `feat/foo` & drops `<remote>` | Parse `(remote, full_branch)` from `refs/remotes/<remote>/<branch>`, push delete to `<remote>` (+ `STEP 3A` remote param) | In scope |
| `P1-23` | P1 | STEP 3C | `src/app.rs:381-1056`, `changes.rs:274`, `stashlist.rs:42`, `stashing.rs:264`, `push.rs:321`, `push_tags.rs:243`, `pull.rs:268`, `commit.rs:583` | Uncaught `?` errors terminate entire TUI | Wrap interactive ops in `try_or_popup!`; handle missing remote gracefully in `delete_tag` | **Add `src/components/changes.rs`** |
| `P1-24` | P1 | STEP 3C | `filetreelist/src/filetreeitems.rs:218,258`, `commitlist.rs:277,711`, `statustree.rs:231-258`, `status_tree.rs:178`, `filetree.rs:213`, `taglist.rs:391`, `blame_file.rs:329,706,788`, `branchlist.rs:590`, `confirm.rs:194`, `remotelist.rs:416-419` | Production underflow, OOB indexing & `.expect()` panics | Replace `- 1` with `checked_sub`/`saturating_sub`, bounds-check `.get()`, remove `.expect()` | **Add `filetreelist` & `components/*` to STEP 3C** |
| `P2-06` | P2 | STEP 3C | `src/popups/reset.rs:112-149`, `src/popups/checkout_option.rs:80-116` | Sticky `Hard`/`DiscardAllLocalChanges` defaults; no secondary confirm; `discard_status` runs before `checkout` | Reset defaults on `open()`, remove wrap-around, add `ConfirmPopup` for destructive choices, emit `NeedsUpdate::ALL` | In scope |
| `P2-07` | P2 | STEP 3C | `src/popups/confirm.rs:131-216`, `src/strings.rs:146-239`, `src/components/diff.rs:547-591,915-942` | Missing targets/untracked warnings in `ConfirmPopup`; `0 selected lines` prompt; `ForcePush` truncates branch | Name target paths/branches/remotes/tags/hashes cleanly; warn on untracked deletion; guard empty `selected_lines` | **Add `src/components/diff.rs`** |
| `P2-08` | P2 | STEP 3C | `revlog.rs:482-522`, `branchlist.rs:348-378`, `commitlist.rs:860-865`, `stashlist.rs:195`, `submodules.rs:186-205`, `status.rs:612-619,899-908`, `remotelist.rs:142-145` | Missing confirmations on revert, merge, rebase, detached-HEAD checkout (leaking to `StashList`), `PushTags`, submodule update; `Undo` during rebase | Add `ConfirmPopup` gates, block checkout in `StashList`, guard `Undo` during rebase, allow force-push when behind | **Add `commitlist.rs`, `queue.rs`** |
| `P2-09` | P2 | STEP 3C | `src/app.rs:316-368,495-572` | Popup Z-order vs event-capture inversion & unblocked app shortcuts | Enforce strict reverse order in `accessors!`/`setup_popups!`; guard tab/options shortcuts with `!any_popup_visible()` | In scope |
| `P2-10` | P2 | STEP 3C | `src/popups/{push.rs:315-326,push_tags.rs:237-247,pull.rs:70-87,249-278,fetch.rs:62-80,162-191}` | `Esc` on cred prompt runs Push/Pull/Fetch anyway; `Pull`/`Fetch` modal lockup on preflight error | Abort & hide on cred cancel (`!is_complete()`); handle `Esc`/`Enter` & hide on error in `Pull`/`Fetch` | In scope |
| `P2-11` | P2 | STEP 3C | `revision_files.rs:123-145`, `taglist.rs:228-260`, `inspect_commit.rs:174-192`, `fuzzy_find.rs:110-129,336-375`, `stashlist.rs:199-221`, `app.rs:803` | Modal event leaks, `FuzzyFind` `Esc`/double-nav bug, `TabSwitchStatus` stale cmdbar | Return `Consumed` in modals & `StashList`; restore/early-return in `FuzzyFind`; update cmdbar on `TabSwitchStatus` | In scope |
| `P2-12` | P2 | STEP 3C | `msg.rs`, `push*.rs`, `pull.rs`, `fetch.rs`, `checkout_option.rs`, `inspect_commit.rs`, `compare_commits.rs`, `file_revlog.rs`, `create_remote.rs`, `remotelist.rs`, `branchlist.rs`, `tag_commit.rs`, `log_search.rs`, `options.rs`, `strings.rs:1592` | 15 popup key-hint/`event()` mismatches, `log_find_commit` wrong key, `cmd_bar_toggle` swallowed | Align hints, `enabled`/`available` flags & `event()` guards; fix `Back [Esc]` on diff popups; pass through `.` | In scope |
| `P2-13` | P2 | STEP 3C | `status_tree.rs:320,485`, `revision_files.rs:488,496`, `tabs/{files.rs:66-101,status.rs:141-142}` | Closing Blame/FileRevlog hides underlying `Files`/`Status`/`Stashing` pane | Remove `self.hide()` back-channel or emit `NeedsUpdate::ALL` / restore visibility on `PopupStackPop` | **Add `status_tree.rs`, `revision_files.rs`** |
| `P2-14` | P2 | STEP 3C | `src/tabs/revlog.rs:128-165,280-290`, `src/tabs/files.rs:50-59` | Search softlock & stuck `(0%)` syntax highlighting when switching tabs | Handle `CommitFilter` & `SyntaxHighlighting` updates even while hidden (or drain on `show()`) | In scope |
| `P2-15` | P2 | STEP 3C | `filetreelist/src/{filetree.rs:70-197,filetreeitems.rs:140-151}`, `app.rs:885-892`, `revision_files.rs:246-251` | `select_file` miss bricks `FilesTab` nav; re-select hidden; sibling collapse icon bug; stale scrollbar | Keep selection on miss, always expand path, check `starts_with` before collapse, update `visual_selection`, scope fuzzy target | **Add `filetreelist/*`, `revision_files.rs`** |
| `P2-16` | P2 | STEP 3C | `commit_details/{details.rs:184-266,mod.rs:147-168}`, `utils/{scroll_vertical.rs,scroll_horizontal.rs}`, `syntax_text.rs:98-137,218-222`, `strings.rs:426-439` | `Details` collapses to 0 rows at 80x24 & clips tags; scroll OOB on resize; `PageUp`/`Down` inverted at 0 height; `ellipsis_trim_start` overflow | Proportional detail height, clamp scroll on resize, guard `visual_height == 0`, reset `SyntaxText` scroll on load, guard `width < 3` | **Add `commit_details/*`, `scroll_*.rs`, `syntax_text.rs`** |
| `P2-17` | P2 | STEP 3C | `src/tabs/stashing.rs:72-80,229-273` | `Stashing` list ignores `stash_untracked` & `keep_index` toggles | Filter `StatusParams` from toggle state & preserve options cleanly | In scope |
| `P2-18` | P2 | STEP 3C | `src/queue.rs:23-35`, `tabs/status.rs:384-899`, `popups/branchlist.rs:321-342,506`, `submodules.rs:364` | `NeedsUpdate::REMOTES = 0b1001` collision; `Status` remote/detached-HEAD bugs; `BranchList` fetch/HEAD/truncate bugs | Set `REMOTES = 0b10000`; call `check_remotes()`; fix `can_fetch`/`can_push`; refresh branches on `Fetch`; assign `.unicode_truncate()` | **Add `src/queue.rs`** |
| `P2-19` | P2 | STEP 3C | `vim_style_key_config.ron:18-19,29`, `textinput.rs:368-396` | Vim `Shift+U` collision; singleline `TextInput` eats `Ctrl+n`/`Ctrl+p` | Gate vertical cursor moves to `Multiline`; resolve vim `Shift+U` mapping cleanly | **Add `textinput.rs`, `vim_style_key_config.ron`** |
| `P2-20` | P2 | STEP 3C | `utils/statustree.rs:55-70,192-207`, `commitlist.rs:93-557`, `utils/mod.rs:49-69` | `StatusTree` binary-searches unsorted items; `CommitList` OOB/mark-slice/byte-width bugs | Linear lookup in `StatusTree`; clamp `CommitList` selection; absolute mark index; `UnicodeWidthStr` widths | **Add `statustree.rs`, `commitlist.rs`, `utils/mod.rs`** |
| `P2-21` | P2 | STEP 3C (+3A) | `fetch.rs:105-114` (`fetch_job.rs`), `blame_file.rs:345-480` (`blame.rs`), `taglist.rs:348-353`, `remotelist.rs:460-471` | Silent fetch/blame/remote-tag/remote-URL failures & infinite blame spinner | Expose `AsyncFetchJob` & `AsyncBlame` errors (`STEP 3A`), show `ShowErrorMsg`, implement `BlameFilePopup::hide()`, handle `Ok(None)` URL | **Add `fetch_job.rs` & `blame.rs` to STEP 3A** |
| `P2-22` | P2 | STEP 3C | `revision_files.rs:133,260-530`, `tabs/{files.rs:36,status.rs:467-480,revlog.rs:464-474}`, `app.rs:182-296,634`, `diff.rs:276`, `keys/{key_list.rs,symbols.rs,key_config.rs}`, `textinput.rs`, `status_tree.rs:147`, `strings.rs:693,759,1328,1772` | Unborn-HEAD hang, invalid persisted tab crash, focus/nav polish, key-parse logging, `Tab` in singleline, hint typos | Clear loading on unborn HEAD/error, clamp persisted tab, fix `Focus::Diff`/`Revlog` `Esc`/key hints, log key-config errors | **Add `revision_files.rs`, `diff.rs`, `keys/*`, `status_tree.rs`, `textinput.rs`** |
| `P2-23` | P2 | STEP 3C | `create_branch.rs`, `rename_branch.rs`, `tag_commit.rs`, `create_remote.rs`, `rename_remote.rs`, `update_remote_url.rs`, `commit.rs`, `stashmsg.rs`, `goto_line.rs`, `submodules.rs` | Missing pre-submit ref/input validation, lost input on error, stale input on `Esc`, raw `{:?}` errors | Validate ref/URL/non-empty inputs before submit, keep popup+text on error, clear on `open()`, format user-friendly statuses | In scope |
| `P3-01` | P3 | STEP 4A | `src/ui/style.rs:12-39,286-290,330-361`, `THEMES.md` | Legacy ANSI `Theme::default()` & pre-0.23 migration hazard | Apply Nebulous/Orrery 7-color P3 Amber-on-Black palette (`#000000`, `#0C0800`, `#FFB000`, `#FFD866`, `#996300`, `#5C3A00`, `#FF5500`); preserve 24 fields + `"¶"`; add `#[serde(default)]`; update `THEMES.md` | In scope |
| `P3-02` | P3 | STEP 4A | `src/ui/style.rs:41-279` | Missing focused border/tab/cmdbar-key/track/info semantic helpers & contrast bug | Implement focused `block(true)`, `tab`, `commandbar_bg`, `commandbar_key`, `commandbar_splitter`, `scroll_bar_track`, `section_header`, `attention_block_style(&self)`, `info_title`; check `text(true, true)` contrast | In scope |
| `P2-24` | P2 | STEP 4B | `src/cmdbar.rs:37-198`, `src/app.rs:351-356,1160-1228`, `popups/{confirm.rs:38,reset.rs:160,checkout_option.rs:130}` | `CommandBar` blank on narrow width, hardcoded `"more [.]"`, missing `r.x +`, popup `.` swallow, cramped confirm rects | Guard `line_width > 0` before `LineBreak`, dynamic toggle hint, include `r.x +`, cap height, allow non-input popup `.`, responsive confirm rects | In scope (`confirm.rs` etc. optional) |
| `P3-03` | P3 | STEP 4B | `src/ui/mod.rs`, `src/ui/scrolllist.rs:68-74`, `src/components/mod.rs:287-301`, `src/popups/mod.rs:88-93` | No shared `Block` helper; mixed `Plain`/`Thick` borders & unpadded titles | Add `ui::styled_block`, `ui::popup_block`, `ui::attention_block` (`BorderType::Plain`, padded `" {title} "`, focused/unfocused tokens) | In scope (+ `popups/mod.rs`) |
| `P3-04` | P3 | STEP 4B | `src/app.rs:1160-1228`, `src/spinner.rs:51-53` | Tab bar/spinner/divider unaligned with Nebulous/Orrery chrome | Apply `AMBER_BRIGHT + BOLD` active tab, `AMBER_DIM` inactive & path, `AMBER_DARK` dividers & rule, themed spinner | **Add `src/spinner.rs`** |
| `P3-05` | P3 | STEP 4B | `src/cmdbar.rs:139-199` | Unstyled splitter, terminal-bg gaps, flat command/key style | Fill full bar surface, split `"Label [key]"` into `commandbar` + `commandbar_key`, style `splitter` & toggle | In scope |
| `P3-06` | P3 | STEP 4B | `components/{syntax_text.rs,diff.rs,commitlist.rs,revision_files.rs,commit_details/*,textinput.rs}`, `tabs/{status.rs,revlog.rs,stashing.rs}`, `ui/scrollbar.rs` | Swapped title/border (`syntax_text`), hardcoded focus/colors, unstyled scrollbar/badges | Migrate all primary panes, badges & scrollbar to `ui::styled_block` & `Theme` helpers | **Add pane/tab/scrollbar files to STEP 4B** |
| `P3-07` | P3 | STEP 4B | `src/popups/{msg,externaleditor,goto_line,fuzzy_find,log_search,help,options,remotelist,submodules,branchlist,...}.rs` | `MsgPopup` info red title, `ExternalEditor` body color hazard, unstyled popups | Apply `Clear` + `PANEL_BG` + `ui::popup_block`, fix `MsgPopup::show_info`, `ExternalEditor`, `FuzzyFind`/`LogSearch` `.style()`, remove raw `Color::Red` | **Add popup chrome files to STEP 4B** |
| `P3-08` | P3 | STEP 4C | `src/snapshots/*` (3 files), `src/gitui.rs:229`, `README.md:265-272` | Snapshot updates & final Linux CI parity | Inspect/accept all 3 `insta` snapshot diffs (confirming `│` filter stays valid), document default theme in `README.md`, pass full CI parity | In scope |
