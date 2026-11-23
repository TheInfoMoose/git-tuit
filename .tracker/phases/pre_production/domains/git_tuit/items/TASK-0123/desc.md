## Problem Description
enter_and_confirm in src/popups/checkout_option.rs tests pops ConfirmAction(CheckoutBranchDiscard) and calls popup.confirm_discard directly, skipping app.rs Action::CheckoutBranchDiscard (confirm_discard + select_branch_popup.update_branches). A regression in that app.rs arm passes every t8b/t11b test. Found during: Phase 11 review (F1).
Found during: Phase 11 review.

## Reproduction Steps
In src/app.rs Action::CheckoutBranchDiscard, delete the confirm_discard call or the update_branches refresh; run cargo test t11b_ t8b_; all pass.

## Fix Log & Agent Notes
