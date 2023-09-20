use armv9a::{define_bitfield, define_bits, define_mask};

// B3.4.16 RmiRecMpidr type
define_bits!(
    MPIDR,
    AFF3[31 - 24],
    AFF2[23 - 16],
    AFF1[15 - 8],
    AFF0[3 - 0]
);

pub fn validate(mpidr: u64) -> bool {
    let must_be_zero = !(MPIDR::AFF0 | MPIDR::AFF1 | MPIDR::AFF2 | MPIDR::AFF3);
    mpidr & must_be_zero == 0
}
