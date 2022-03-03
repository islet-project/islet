use crate::aarch64::MPIDR_EL1;
use crate::config::NUM_OF_CPU_PER_CLUSTER;

#[naked]
#[no_mangle]
pub unsafe extern "C" fn get_cpu_id() -> usize {
    #![allow(unsupported_naked_functions)]
    let (cluster, core) = id();
    cluster * NUM_OF_CPU_PER_CLUSTER + core
}

#[inline(always)]
pub unsafe fn id() -> (usize, usize) {
    (
        MPIDR_EL1.get_masked_value(MPIDR_EL1::AFF2) as usize,
        MPIDR_EL1.get_masked_value(MPIDR_EL1::AFF1) as usize,
    )
}
