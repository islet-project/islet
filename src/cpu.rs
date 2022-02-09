use crate::config;

pub const MPIDR_AFF1_MASK: usize = 0x00ff00;
pub const MPIDR_AFF2_MASK: usize = 0xff0000;

#[naked]
#[no_mangle]
pub unsafe extern "C" fn get_cpu_id() {
    #![allow(unsupported_naked_functions)]
    llvm_asm! {
        "
        mrs x0, mpidr_el1
        and x1, x0, $0
        and x0, x0, $1
        lsr x0, x0, #8
        lsr x1, x1, #16
        mov x2, $2
        mul x1, x1, x2
        add x0, x0, x1
        ret
        "
        : : "i"(MPIDR_AFF2_MASK), "i"(MPIDR_AFF1_MASK),
            "i"(config::NUM_OF_CPU_PER_CLUSTER) : : "volatile"
    }
}

#[inline(always)]
pub fn id() -> (usize, usize) {
    let id: usize;
    unsafe {
        llvm_asm! {
            "
            bl get_cpu_id
            "
            : "={x0}"(id) : : "x1", "x2" : "volatile"
        }
    }

    (
        id / config::NUM_OF_CPU_PER_CLUSTER,
        id % config::NUM_OF_CPU_PER_CLUSTER,
    )
}
