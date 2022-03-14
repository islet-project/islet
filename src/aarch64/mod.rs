#[macro_use]
pub mod r#macro;

pub mod asm;
pub mod cpu;
pub mod regs;
pub mod trap;

use crate::hyper::vcpu::VCPU;
use realm_management_monitor::{io::Write as IoWrite, println};
pub use regs::*;

global_asm!(include_str!("vectors.s"));
extern "C" {
    static mut vectors: u64;
}

unsafe fn enable_hyp_mode() {
    HCR_EL2.set(
        HCR_EL2::RW, // Execution state for EL1 is AArch64
                     // | HCR_EL2::VM  // Enable stage 2 address translation
                     // | HCR_EL2::TSC // Traps SMC instructions
                     // | HCR_EL2::FMO // Route physical FIQ interrupts to EL2
    );
}

pub unsafe fn init() {
    println!(
        "[Core{:2}] CurrentEL is {}",
        cpu::get_cpu_id(),
        regs::current_el()
    );

    enable_hyp_mode();

    VCPU::vcpu_init();

    VBAR_EL2.set(&vectors as *const u64 as u64);

    asm::brk(10);
}
