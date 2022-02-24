#[macro_use]
pub mod r#macro;

pub mod asm;
pub mod regs;
pub mod trap;

use realm_management_monitor::{io::Write as IoWrite, println};
pub use regs::*;

global_asm!(include_str!("vectors.s"));
extern "C" {
    static mut vectors: u64;
}

pub unsafe fn init() {
    VBAR_EL2.set(&vectors as *const u64 as u64);

    println!("CurrentEL is {}", regs::current_el());
    asm::brk(10);
}
