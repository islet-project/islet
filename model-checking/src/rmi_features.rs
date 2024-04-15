use islet_rmm::monitor::Monitor;
use islet_rmm::rmi;

#[kani::proof]
#[kani::unwind(4)]
fn verify_features() {
    // Initialize registers (symbolic input)
    let regs: [usize; 8] = kani::any();
    kani::assume(regs[0] == rmi::FEATURES);
    // TODO: check the below again
    let index = regs[1];

    // Pre-conditions
    let no_failures_pre = true;
    let success_index_pre = index != 0;

    // Execute command and read the result.
    let out = Monitor::new().run(regs);
    let result = out[0];
    let value = out[1];

    // Result assertion
    let prop_result_ante = no_failures_pre;

    kani::cover!();
    if prop_result_ante {
        let prop_result_cons = result == rmi::SUCCESS;

        kani::cover!();
        assert!(prop_result_cons);
    }

    // Success condition assertions
    let prop_success_index_ante = no_failures_pre && success_index_pre;

    kani::cover!();
    if prop_success_index_ante {
        let success_index_post = value == 0;
        let prop_success_index_cons = success_index_post && (result == rmi::SUCCESS);

        kani::cover!();
        assert!(prop_success_index_cons);
    }
}
