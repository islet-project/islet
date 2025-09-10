// SPDX-License-Identifier: Apache-2.0 OR MIT
//

//! AArch64 Processor Feature Register 1 - EL1
//!
//! Provides additional information about implemented PE features in AArch64 state.

use tock_registers::{interfaces::Readable, register_bitfields};

register_bitfields! {u64,
    pub ID_AA64ZFR0_EL1 [
        /// Support for SVE FP64 double-precision floating-point matrix multiplication instructions
        F64MM OFFSET(56) NUMBITS(4) [],
        /// Support for the SVE FP32 single-precision floating-point matrix multiplication instruction
        F32MM OFFSET(52) NUMBITS(4) [],
        /// Support for SVE Int8 matrix multiplication instructions
        I8MM OFFSET(44) NUMBITS(4) [],
        /// Support for SVE SM4 instructions
        SM4 OFFSET(40) NUMBITS(4) [],
        /// Support for the SVE SHA3 instructions
        SHA3 OFFSET(32) NUMBITS(4) [],
        /// Support for SVE BFloat16 instructions
        BF16 OFFSET(20) NUMBITS(4) [],
        /// Support for SVE bit permute instructions
        BitPerm OFFSET(16) NUMBITS(4) [],
        /// Support for SVE AES instructions
        AES OFFSET(4) NUMBITS(4) [],
        /// Support for SVE
        SVEver OFFSET(0) NUMBITS(4) [],
    ]
}

pub struct Reg;

impl Readable for Reg {
    type T = u64;
    type R = ID_AA64ZFR0_EL1::Register;

    sys_coproc_read_raw!(u64, "ID_AA64PFR1_EL1", "x");
}

pub const ID_AA64ZFR0_EL1: Reg = Reg {};
