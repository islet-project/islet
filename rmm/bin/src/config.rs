pub const ABI_VERSION: usize = 1;

pub const NUM_OF_CPU: usize = 8;
pub const NUM_OF_CLUSTER: usize = 2;
pub const NUM_OF_CPU_PER_CLUSTER: usize = NUM_OF_CPU / NUM_OF_CLUSTER;

pub const PAGE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_BITS;

pub const RMM_STACK_SIZE: usize = 1024 * 1024;
pub const RMM_HEAP_SIZE: usize = 8 * 1024 * 1024;

pub const VM_STACK_SIZE: usize = 1 << 20;
pub const STACK_ALIGN: usize = 16;
