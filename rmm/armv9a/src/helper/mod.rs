#[macro_use]
pub mod r#macro;

pub mod asm;
pub mod regs;

use monitor::{io::Write as IoWrite, println};
pub use regs::*;

use crate::cpu;
use crate::exception::vectors;

pub const fn bits_in_reg(mask: u64, val: u64) -> u64 {
    (val << (mask.trailing_zeros())) & mask
}

fn activate_stage2_mmu() {
    // stage 2 intitial table: L1 with 1024 entries (2 continuous 4KB pages)
    let vtcr_el2: u64 = bits_in_reg(VTCR_EL2::PS, tcr_paddr_size::PS_1T)
        | bits_in_reg(VTCR_EL2::TG0, tcr_granule::G_4K)
        | bits_in_reg(VTCR_EL2::SH0, tcr_shareable::INNER)
        | bits_in_reg(VTCR_EL2::ORGN0, tcr_cacheable::NONE)
        | bits_in_reg(VTCR_EL2::IRGN0, tcr_cacheable::NONE)
        | bits_in_reg(VTCR_EL2::SL0, tcr_start_level::L1)
        | bits_in_reg(VTCR_EL2::T0SZ, 24); // T0SZ, input address is 2^40 bytes

    unsafe {
        // Flush dcache
        // Invalidate the local I-cache so that any instructions fetched
        // speculatively are discarded.
        //dcache cisw
        llvm_asm! {
            "
            ic iallu
            dsb nsh
            isb
            " : : : :
        }

        VTCR_EL2.set(vtcr_el2);

        llvm_asm! {
            "
            tlbi alle2
            dsb ish
            isb
            " : : : :
        }
    }
}

pub unsafe fn init() {
    println!(
        "[Core{:2}] CurrentEL is {}",
        cpu::get_cpu_id(),
        regs::current_el()
    );

    HCR_EL2.set(HCR_EL2::RW | HCR_EL2::TSC | HCR_EL2::VM);
    VBAR_EL2.set(&vectors as *const u64 as u64);
    SCTLR_EL2.set(SCTLR_EL2::I | SCTLR_EL2::C);
    activate_stage2_mmu();

    // asm::brk(10);
}

pub unsafe fn rmm_exit(args: [usize; 3]) -> [usize; 3] {
    let mut ret: [usize; 3] = [0usize; 3];

    llvm_asm! {
        "bl rmm_exit"
        : "={x0}"(ret[0]), "={x1}"(ret[1]), "={x2}"(ret[2])
        : "{x0}"(args[0]), "{x1}"(args[1]), "{x2}"(args[2])
        : : "volatile"
    }
    ret
}

// extern "C" {
//     pub fn rmm_exit(arg0: usize, arg1: usize, arg2: usize) -> [u64; 3];
// }
