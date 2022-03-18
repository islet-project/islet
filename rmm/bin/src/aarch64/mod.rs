#[macro_use]
pub mod r#macro;

pub mod asm;
pub mod cpu;
pub mod regs;
pub mod trap;

use crate::realm;
pub use regs::*;
use rmm_core::{io::Write as IoWrite, println};

global_asm!(include_str!("vectors.s"));
extern "C" {
    static mut vectors: u64;
    fn restore_all_from_vcpu_and_run();
}

pub unsafe fn init() {
    println!(
        "[Core{:2}] CurrentEL is {}",
        cpu::get_cpu_id(),
        regs::current_el()
    );

    VBAR_EL2.set(&vectors as *const u64 as u64);

    realm::registry::get(0).unwrap().lock().vcpus[cpu::get_cpu_id()]
        .lock()
        .set_current();

    // asm::brk(10);

    restore_all_from_vcpu_and_run();
}
