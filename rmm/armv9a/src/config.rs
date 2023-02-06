pub const ABI_VERSION: usize = 1;

pub const NUM_OF_CPU: usize = 8;
pub const NUM_OF_CLUSTER: usize = 2;
pub const NUM_OF_CPU_PER_CLUSTER: usize = NUM_OF_CPU / NUM_OF_CLUSTER;

pub const PAGE_BITS: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_BITS; // 4KiB
pub const LARGE_PAGE_SIZE: usize = 1024 * 1024 * 2; // 2MiB
pub const HUGE_PAGE_SIZE: usize = 1024 * 1024 * 1024; // 1GiB

pub const RMM_STACK_SIZE: usize = 1024 * 1024;
pub const RMM_HEAP_SIZE: usize = 8 * 1024 * 1024;

pub const VM_STACK_SIZE: usize = 1 << 15;
pub const STACK_ALIGN: usize = 16;

// defined in trusted-firmware-a/include/services/rmmd_svc.h
pub const SMC_ASC_MARK_REALM: usize = 0xc400_01b0;
pub const SMC_ASC_MARK_NONSECURE: usize = 0xc400_01b1;

pub enum Code {
    MarkRealm,
    MarkNonSecure,
}

impl From<Code> for usize {
    fn from(origin: Code) -> Self {
        match origin {
            Code::MarkRealm => SMC_ASC_MARK_REALM,
            Code::MarkNonSecure => SMC_ASC_MARK_NONSECURE,
        }
    }
}
