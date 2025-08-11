use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

pub const NUM_OF_CPU: usize = 8;
pub const NUM_OF_CLUSTER: usize = 2;
pub const NUM_OF_CPU_PER_CLUSTER: usize = NUM_OF_CPU / NUM_OF_CLUSTER;

pub const PAGE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_BITS; // 4KiB
pub const LARGE_PAGE_SIZE: usize = 1024 * 1024 * 2; // 2MiB
pub const HUGE_PAGE_SIZE: usize = 1024 * 1024 * 1024; // 1GiB

#[cfg(any(feature = "fvp", not(feature = "qemu")))]
pub const MAX_DRAM_SIZE: usize = 0xFC00_0000; // 4GB - 64MB
#[cfg(feature = "qemu")]
pub const MAX_DRAM_SIZE: usize = 0x2_0000_0000; // 8GB

pub const RMM_STACK_GUARD_SIZE: usize = crate::granule::GRANULE_SIZE * 1;
pub const RMM_STACK_SIZE: usize = 1024 * 1024 - RMM_STACK_GUARD_SIZE;
pub const RMM_HEAP_SIZE: usize = 16 * 1024 * 1024;

pub const VM_STACK_SIZE: usize = 1 << 15;
pub const STACK_ALIGN: usize = 16;

pub const SMCCC_1_3_SVE_HINT: usize = 1 << 16;

#[derive(Debug, Default)]
pub struct PlatformMemoryLayout {
    pub rmm_base: u64,
    pub rw_start: u64,
    pub rw_end: u64,
    pub stack_base: u64,
    pub uart_phys: u64,
    pub el3_shared_buf: u64,
}

lazy_static! {
    pub static ref NS_DRAM_REGIONS: Mutex<Vec<core::ops::Range<usize>>> = Mutex::new(Vec::new());
}

pub fn is_ns_dram(addr: usize) -> bool {
    let regions = NS_DRAM_REGIONS.lock();

    for range in regions.iter() {
        if range.contains(&addr) {
            return true;
        }
    }

    false
}
