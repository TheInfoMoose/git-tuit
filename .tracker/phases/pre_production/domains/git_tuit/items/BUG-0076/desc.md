## Problem Description
Test tabs::status::tests::t7j_focus_leaves_only_loaded_empty_diff (src/tabs/status.rs:1028-1078) intermittently panics at line 1070 (assertion failed: focused_while_loading) or line 1074 (assertion failed: focused_while_loading_wd). Root cause: the test inserts a synthetic StatusItem (staged.txt / workdir.txt) and immediately calls switch_focus(Focus::Stage|WorkDir) followed by switch_focus(Focus::Diff), assuming the background asyncgit::diff worker is still pending (pending == true). When the worker finishes computing the empty diff before switch_focus(Focus::Diff) runs, pending is already false so focus leaves the empty diff and focused_while_loading is false. Needs deterministic synchronization or decoupling from background worker timing (and temporarily marking the test #[ignore] if needed outside STEP 8A).

## Reproduction Steps
1. Inspect `.`.
2. Run tests to verify behavior.

## Fix Log & Agent Notes
- Target scope path: `.`
