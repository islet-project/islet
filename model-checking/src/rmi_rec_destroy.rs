use crate::common::addr_is_granule_aligned;
use crate::common::initialize;
use crate::common::post_granule_state;
use crate::common::post_rec_aux_state;
use crate::common::pre_granule_state;
use crate::common::pre_rec_state;
use islet_rmm::granule::validate_addr;
use islet_rmm::granule::GranuleState;
use islet_rmm::monitor::Monitor;
use islet_rmm::rec::State;
use islet_rmm::rmi;
use islet_rmm::rmi::error::Error;

#[kani::proof]
#[kani::unwind(7)]
fn verify_rec_destroy() {
    initialize();

    // Initialize registers (symbolic input)
    let regs: [usize; 8] = kani::any();
    kani::assume(regs[0] == rmi::REC_DESTROY);
    // TODO: check the below again
    let rec = regs[1];

    // Pre-conditions
    let failure_rec_align_pre = !addr_is_granule_aligned(rec);
    let failure_rec_bound_pre = !validate_addr(rec);
    let failure_rec_gran_state_pre = pre_granule_state(rec) != GranuleState::Rec;
    let failure_rec_state_pre = pre_rec_state(rec) == State::Running;
    let no_failures_pre = !failure_rec_align_pre
        && !failure_rec_bound_pre
        && !failure_rec_gran_state_pre
        && !failure_rec_state_pre;

    // Execute command and read the result.
    let out = Monitor::new().run(regs);
    let result = out[0];

    // Failure condition assertions
    let prop_failure_rec_align_ante = failure_rec_align_pre;

    kani::cover!();
    if prop_failure_rec_align_ante {
        let failure_rec_align_post = result == Error::RmiErrorInput.into();
        let prop_failure_rec_align_cons = failure_rec_align_post;

        kani::cover!();
        assert!(prop_failure_rec_align_cons);
    }

    let prop_failure_rec_bound_ante = !failure_rec_align_pre && failure_rec_bound_pre;

    kani::cover!();
    if prop_failure_rec_bound_ante {
        let failure_rec_bound_post = result == Error::RmiErrorInput.into();
        let prop_failure_rec_bound_cons = failure_rec_bound_post;

        kani::cover!();
        assert!(prop_failure_rec_bound_cons);
    }

    let prop_failure_rec_gran_state_ante =
        !failure_rec_align_pre && !failure_rec_bound_pre && failure_rec_gran_state_pre;

    kani::cover!();
    if prop_failure_rec_gran_state_ante {
        let failure_rec_gran_state_post = result == Error::RmiErrorInput.into();
        let prop_failure_rec_gran_state_cons = failure_rec_gran_state_post;

        kani::cover!();
        assert!(prop_failure_rec_gran_state_cons);
    }

    let prop_failure_rec_state_ante = !failure_rec_align_pre
        && !failure_rec_bound_pre
        && !failure_rec_gran_state_pre
        && failure_rec_state_pre;

    kani::cover!();
    if prop_failure_rec_state_ante {
        let failure_rec_state_post = result == Error::RmiErrorRec.into();
        let prop_failure_rec_state_cons = failure_rec_state_post;

        kani::cover!();
        assert!(prop_failure_rec_state_cons);
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
    let prop_success_rec_gran_state_ante = no_failures_pre;

    kani::cover!();
    if prop_success_rec_gran_state_ante {
        let success_rec_gran_state_post = post_granule_state(rec) == GranuleState::Delegated;
        let prop_success_rec_gran_state_cons =
            success_rec_gran_state_post && result == rmi::SUCCESS;

        kani::cover!();
        assert!(prop_success_rec_gran_state_cons);
    }

    let prop_success_rec_aux_state_ante = no_failures_pre;

    kani::cover!();
    if prop_success_rec_aux_state_ante {
        let success_rec_aux_state_post = post_rec_aux_state(rec) == GranuleState::Delegated;
        let prop_success_rec_aux_state_cons = success_rec_aux_state_post && result == rmi::SUCCESS;

        kani::cover!();
        assert!(prop_success_rec_aux_state_cons);
    }
}
