use islet_rmm::monitor::Monitor;

#[kani::proof]
#[kani::unwind(4)]
fn verify_features() {
    Monitor::new().run();
    kani::cover!();
}
