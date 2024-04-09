use islet_rmm::monitor::Monitor;
use islet_rmm::rmi;

#[kani::proof]
#[kani::unwind(4)]
fn verify_version() {
    // Initialize registers (symbolic input)
    let regs: [usize; 8] = kani::any();
    kani::assume(regs[0] == rmi::VERSION);

    // Pre-conditions
    let no_failures_pre = true;

    // Execute command and read the result.
    let out = Monitor::new().run(regs);
    let result = out[0];
    let lower = out[1];
    let higher = out[2];

    // Result assertion
    let prop_result_ante = no_failures_pre;

    kani::cover!();
    if prop_result_ante {
        let prop_result_cons = (lower == (rmi::ABI_MAJOR_VERSION << 16) | rmi::ABI_MINOR_VERSION)
            && (higher == (rmi::ABI_MAJOR_VERSION << 16) | rmi::ABI_MINOR_VERSION);

        kani::cover!();
        assert!(prop_result_cons);
    }

    kani::cover!();
}
