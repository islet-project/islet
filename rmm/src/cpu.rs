use crate::config::NUM_OF_CPU_PER_CLUSTER;

use aarch64_cpu::registers::*;

#[no_mangle]
pub extern "C" fn get_cpu_id() -> usize {
    let (cluster, core) = id();
    cluster * NUM_OF_CPU_PER_CLUSTER + core
}

#[inline(always)]
pub fn id() -> (usize, usize) {
    (
        MPIDR_EL1.read(MPIDR_EL1::Aff2) as usize,
        MPIDR_EL1.read(MPIDR_EL1::Aff1) as usize,
    )
}
