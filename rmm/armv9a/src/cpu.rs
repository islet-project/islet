use crate::config::NUM_OF_CPU_PER_CLUSTER;
use crate::helper::MPIDR_EL1;

#[naked]
#[no_mangle]
pub extern "C" fn get_cpu_id() -> usize {
    #![allow(unsupported_naked_functions)]
    let (cluster, core) = id();
    cluster * NUM_OF_CPU_PER_CLUSTER + core
}

#[inline(always)]
pub fn id() -> (usize, usize) {
    unsafe {
        (
            MPIDR_EL1.get_masked_value(MPIDR_EL1::AFF2) as usize,
            MPIDR_EL1.get_masked_value(MPIDR_EL1::AFF1) as usize,
        )
    }
}
