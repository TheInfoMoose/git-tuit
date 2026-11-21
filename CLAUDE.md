# git-tuit

Fork of gitui (Rust TUI for git). Keep the `gitui` binary name and `~/.config/gitui/` config compatibility.

## Workflow
- **Phased workflow** (`phased-workflow` skill): the plan header in `STEPPED_IMPLEMENTATION_PLAN.md` says `**Workflow:** phased`. Claude writes phases and reviews them; Gemini runs them. Claude never builds, tests or edits source here.
- **neb tracking:** yes (`.tracker/`). Every audit finding in `AUDIT_REPORT.md` has a neb item; `.phased/finding-items.json` maps finding ID → item ID.
- **Orrery:** yes, profile `gemini-phased` (`agy-hydrogen-orch` → `agy-neon-impl` → `agy-hydrogen-val`, Loop mode). Repo rules for agents: `.orrery/skills/repo-rules.md`.

## Gates
Base gate lives in the plan header. Target end state: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`, plus `cargo deny check`.
