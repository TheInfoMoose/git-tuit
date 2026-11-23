## Problem Description
The t10c popup tests call discard_finished directly; the real path (Enter on Discard -> ConfirmAction -> app.rs Action::CheckoutBranchDiscard -> confirm_discard) is untested, so dropping the outcome in confirm_discard would pass every test while the user never sees the merge-state warning.
Found during: Phase 10 review.

## Reproduction Steps
In src/popups/checkout_option.rs change confirm_discard to ignore the DiscardOutcome (push Update and hide only); run cargo test t10c_ and all pass.

## Fix Log & Agent Notes
