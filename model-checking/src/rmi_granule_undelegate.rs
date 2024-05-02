use crate::common::addr_is_granule_aligned;
use crate::common::initialize;
use crate::common::{post_granule_gpt, post_granule_state, pre_granule_state};
use crate::get_granule;
use islet_rmm::granule::entry::GranuleGpt;
use islet_rmm::granule::validate_addr;
use islet_rmm::granule::GranuleState;
use islet_rmm::monitor::Monitor;
use islet_rmm::rmi;
use islet_rmm::rmi::error::Error;

#[kani::proof]
#[kani::unwind(9)]
fn verify_granule_undelegate() {
    initialize();

    // Initialize registers (symbolic input)
    let regs: [usize; 8] = kani::any();
    kani::assume(regs[0] == rmi::GRANULE_UNDELEGATE);
    // TODO: check the below again
    let addr = regs[1];

    // Pre-conditions
    let failure_gran_align_pre = !addr_is_granule_aligned(addr);
    let failure_gran_bound_pre = !validate_addr(addr);
    let failure_gran_state_pre = pre_granule_state(addr) != GranuleState::Delegated;
    let no_failures_pre =
        !failure_gran_align_pre && !failure_gran_bound_pre && !failure_gran_state_pre;

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

    // Result assertion
    let prop_result_ante = no_failures_pre;

    kani::cover!();
    if prop_result_ante {
        let prop_result_cons = result == rmi::SUCCESS;

        kani::cover!();
        assert!(prop_result_cons);
    }

    // Success condition assertions
    let prop_success_gran_gpt_ante = no_failures_pre;

    kani::cover!();
    if prop_success_gran_gpt_ante {
        let success_gran_gpt_post = post_granule_gpt(addr) == GranuleGpt::GPT_NS;
        let prop_success_gran_gpt_cons = success_gran_gpt_post && (result == rmi::SUCCESS);

        kani::cover!();
        assert!(prop_success_gran_gpt_cons);
    }

    let prop_success_gran_state_ante = no_failures_pre;

    kani::cover!();
    if prop_success_gran_state_ante {
        let success_gran_state_post = post_granule_state(addr) == GranuleState::Undelegated;
        let prop_success_gran_state_cons = success_gran_state_post && (result == rmi::SUCCESS);

        kani::cover!();
        assert!(prop_success_gran_state_cons);
    }

    let prop_success_gran_content_ante = no_failures_pre;

    kani::cover!();
    if prop_success_gran_content_ante {
        let success_gran_content = get_granule!(addr)
            .map(|guard| guard.index_to_addr())
            .unwrap();
        // check the first byte to reduce the proof overhead
        let success_gran_content_post = unsafe { *(success_gran_content as *const u8) } == 0;

        let prop_success_gran_content_cons = success_gran_content_post && (result == rmi::SUCCESS);

        kani::cover!();
        assert!(prop_success_gran_content_cons);
    }
}
