use crate::common::addr_is_granule_aligned;
use crate::common::initialize;
use crate::common::{post_rd_state, pre_granule_state, pre_rd_state};
use crate::get_granule;
use islet_rmm::granule::validate_addr;
use islet_rmm::granule::GranuleState;
use islet_rmm::monitor::Monitor;
use islet_rmm::realm::rd::Rd;
use islet_rmm::realm::rd::State;
use islet_rmm::rmi;
use islet_rmm::rmi::error::Error;

#[kani::proof]
#[kani::unwind(9)]
fn verify_realm_activate() {
    initialize();

    // Initialize registers (symbolic input)
    let regs: [usize; 8] = kani::any();
    kani::assume(regs[0] == rmi::REALM_ACTIVATE);
    // TODO: check the below again
    let rd = regs[1];

    // Pre-conditions
    let failure_rd_align_pre = !addr_is_granule_aligned(rd);
    let failure_rd_bound_pre = !validate_addr(rd);
    let failure_rd_state_pre = pre_granule_state(rd) != GranuleState::RD;
    let failure_realm_state_pre = pre_rd_state(rd) != State::New;
    let no_failures_pre = !failure_rd_align_pre
        && !failure_rd_bound_pre
        && !failure_rd_state_pre
        && !failure_realm_state_pre;

    // Execute command and read the result.
    let out = Monitor::new().run(regs);
    let result = out[0];

    // Failure condition assertions
    let prop_failure_rd_align_ante = failure_rd_align_pre;

    kani::cover!();
    if prop_failure_rd_align_ante {
        let failure_rd_align_post = result == Error::RmiErrorInput.into();
        let prop_failure_rd_align_cons = failure_rd_align_post;

        kani::cover!();
        assert!(prop_failure_rd_align_cons);
    }

    let prop_failure_rd_bound_ante = !failure_rd_align_pre && failure_rd_bound_pre;

    kani::cover!();
    if prop_failure_rd_bound_ante {
        let failure_rd_bound_post = result == Error::RmiErrorInput.into();
        let prop_failure_rd_bound_cons = failure_rd_bound_post;

        kani::cover!();
        assert!(prop_failure_rd_bound_cons);
    }

    let prop_failure_rd_state_ante =
        !failure_rd_align_pre && !failure_rd_bound_pre && failure_rd_state_pre;

    kani::cover!();
    if prop_failure_rd_state_ante {
        let failure_rd_state_post = result == Error::RmiErrorInput.into();
        let prop_failure_rd_state_cons = failure_rd_state_post;

        kani::cover!();
        assert!(prop_failure_rd_state_cons);
    }

    let prop_failure_realm_state_ante = !failure_rd_align_pre
        && !failure_rd_bound_pre
        && !failure_rd_state_pre
        && failure_realm_state_pre;

    kani::cover!();
    if prop_failure_realm_state_ante {
        let failure_realm_state_post = result == Error::RmiErrorRealm(0).into();
        let prop_failure_realm_state_cons = failure_realm_state_post;

        kani::cover!();
        assert!(prop_failure_realm_state_cons);
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
    let prop_success_realm_state_ante = no_failures_pre;

    kani::cover!();
    if prop_success_realm_state_ante {
        let success_realm_state_post = post_rd_state(rd) == State::Active;
        let prop_success_realm_state_cons = success_realm_state_post && result == rmi::SUCCESS;

        kani::cover!();
        assert!(prop_success_realm_state_cons);
    }
}
