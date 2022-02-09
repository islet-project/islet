pub const NUM_OF_CPU: usize = 8;
pub const NUM_OF_CLUSTER: usize = 2;
pub const NUM_OF_CPU_PER_CLUSTER: usize = NUM_OF_CPU / NUM_OF_CLUSTER;

pub const RMM_STACK_SIZE: usize = 0x2000; //8192
pub const RMM_HEAP_SIZE: usize = 0x10000; //1024 * 1024
