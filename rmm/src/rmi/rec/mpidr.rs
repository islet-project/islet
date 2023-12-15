use armv9a::{define_bitfield, define_bits, define_mask};

// B3.4.16 RmiRecMpidr type
define_bits!(
    MPIDR,
    AFF3[31 - 24],
    AFF2[23 - 16],
    AFF1[15 - 8],
    AFF0[3 - 0]
);

impl From<u64> for MPIDR {
    fn from(val: u64) -> Self {
        Self(val)
    }
}

impl MPIDR {
    // B2.30 RecIndex function
    pub fn index(&self) -> usize {
        let aff0 = self.get_masked_value(MPIDR::AFF0) as usize;
        let aff1 = self.get_masked_value(MPIDR::AFF1) as usize;
        let aff2 = self.get_masked_value(MPIDR::AFF2) as usize;
        let aff3 = self.get_masked_value(MPIDR::AFF3) as usize;

        aff0 + (16 * aff1) + (16 * 256 * aff2) + (16 * 256 * 256 * aff3)
    }
}

pub fn validate(mpidr: u64) -> bool {
    let must_be_zero = !(MPIDR::AFF0 | MPIDR::AFF1 | MPIDR::AFF2 | MPIDR::AFF3);
    mpidr & must_be_zero == 0
}
