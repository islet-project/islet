use crate::common::addr_is_granule_aligned;
use crate::common::{post_granule_gpt, post_granule_state, pre_granule_gpt, pre_granule_state};
use crate::get_granule;
use islet_rmm::granule::entry::GranuleGpt;
use islet_rmm::granule::validate_addr;
use islet_rmm::granule::GranuleState;
use islet_rmm::monitor::Monitor;
use islet_rmm::rmi;
use islet_rmm::rmi::error::Error;

#[kani::proof]
#[kani::unwind(9)]
fn verify_granule_delegate() {
    // Initialize registers (symbolic input)
    let regs: [usize; 8] = kani::any();
    kani::assume(regs[0] == rmi::GRANULE_DELEGATE);
    // TODO: check the below again
    let addr = regs[1];

    // Pre-conditions
    let failure_gran_align_pre = !addr_is_granule_aligned(addr);
    let failure_gran_bound_pre = !validate_addr(addr);
    let failure_gran_state_pre = pre_granule_state(addr) != GranuleState::Undelegated;
    let failure_gran_gpt_pre = pre_granule_gpt(addr) != GranuleGpt::GPT_NS;
    let no_failures_pre = !failure_gran_align_pre
        && !failure_gran_bound_pre
        && !failure_gran_state_pre
        && !failure_gran_gpt_pre;

    // Execute command and read the result.
    let out = Monitor::new().run(regs);
    let result = out[0];

    // Post conditions have been moved to be inside if statements
    // to constrain the conditions in order not to touch the panic in unwrap();

    // Failure condition assertions
    let prop_failure_gran_align_ante = failure_gran_align_pre;

    kani::cover!();
    if prop_failure_gran_align_ante {
        let failure_gran_align_post = result == Error::RmiErrorInput.into();
        let prop_failure_gran_align_cons = failure_gran_align_post;

        kani::cover!();
        assert!(prop_failure_gran_align_cons);
    }

    let prop_failure_gran_bound_ante = !failure_gran_align_pre && failure_gran_bound_pre;

    kani::cover!();
    if prop_failure_gran_bound_ante {
        let failure_gran_bound_post = result == Error::RmiErrorInput.into();
        let prop_failure_gran_bound_cons = failure_gran_bound_post;

        kani::cover!();
        assert!(prop_failure_gran_bound_cons);
    }

    let prop_failure_gran_state_ante =
        !failure_gran_align_pre && !failure_gran_bound_pre && failure_gran_state_pre;

    kani::cover!();
    if prop_failure_gran_state_ante {
        let failure_gran_state_post = result == Error::RmiErrorInput.into();
        let prop_failure_gran_state_cons = failure_gran_state_post;

        kani::cover!();
        assert!(prop_failure_gran_state_cons);
    }

    let prop_failure_gran_gpt_ante = !failure_gran_align_pre
        && !failure_gran_bound_pre
        && !failure_gran_state_pre
        && failure_gran_gpt_pre;

    kani::cover!();
    if prop_failure_gran_gpt_ante {
        let failure_gran_gpt_post = result == Error::RmiErrorInput.into();
        let prop_failure_gran_gpt_cons = failure_gran_gpt_post;

        kani::cover!();
        assert!(prop_failure_gran_gpt_cons);
    }

    // Result assertion
    let prop_result_ante = no_failures_pre;

    kani::cover!();
    if prop_result_ante {
        let prop_result_cons = result == rmi::SUCCESS;

        kani::cover!();
        assert!(prop_result_cons);
    }

    // Success condition assertions
    let prop_success_gran_state_ante = no_failures_pre;

    kani::cover!();
    if prop_success_gran_state_ante {
        let success_gran_state_post = post_granule_state(addr) == GranuleState::Delegated;
        let prop_success_gran_state_cons = success_gran_state_post && (result == rmi::SUCCESS);

        kani::cover!();
        assert!(prop_success_gran_state_cons);
    }

    let prop_success_gran_gpt_ante = no_failures_pre;

    kani::cover!();
    if prop_success_gran_gpt_ante {
        let success_gran_gpt_post = post_granule_gpt(addr) == GranuleGpt::GPT_REALM;
        let prop_success_gran_gpt_cons = success_gran_gpt_post && (result == rmi::SUCCESS);

        kani::cover!();
        assert!(prop_success_gran_gpt_cons);
    }
}
