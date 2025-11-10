//! Streaming Vector Control Register
//!
//! Controls Streaming SVE mode and SME behavior.

use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields,
};

register_bitfields! {u64,
    pub SVCR [
        /// Enables SME ZA storage.
        ZA OFFSET(1) NUMBITS(1) [],
        /// Enables Streaming SVE mode.
        SM OFFSET(0) NUMBITS(1) [],
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = SVCR::Register;

    // Use the opcode instead of its register mnemonic
    // to pass compilation without SIMD(neon, sve, sme) features in the compile option
    //sys_coproc_read_raw!(u64, "SVCR", "x");
    sys_coproc_read_raw!(u64, "S3_3_C4_C2_2", "x");
}

impl Writeable for Reg {
    type T = u64;
    type R = SVCR::Register;

    //sys_coproc_write_raw!(u64, "SVCR", "x");
    sys_coproc_write_raw!(u64, "S3_3_C4_C2_2", "x");
}
pub const SVCR: Reg = Reg {};
