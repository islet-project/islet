//! SME Control Register - EL2

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub SMCR_EL2 [
        /// When FEAT_SME_FA64 is implemented:
        /// Controls whether execution of an A64 instruction is
        /// considered legal when the PE is in Streaming SVE mode
        FA64 OFFSET(31) NUMBITS(1) [],
        /// Reserved
        RAZWI   OFFSET(4) NUMBITS(5) [],
        /// Effective Streaming SVE Vector Length (SVL)
        LEN OFFSET(0) NUMBITS(4) []
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = SMCR_EL2::Register;

    // Use the opcode instead of its register mnemonic
    // to pass compilation without SIMD(neon, sve, sme) features in the compile option
    //sys_coproc_read_raw!(u64, "SMCR_EL2", "x");
    sys_coproc_read_raw!(u64, "S3_4_C1_C2_6", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = SMCR_EL2::Register;

    //sys_coproc_write_raw!(u64, "SMCR_EL2", "x");
    sys_coproc_write_raw!(u64, "S3_4_C1_C2_6", "x");
}

pub const SMCR_EL2: Reg = Reg {};
