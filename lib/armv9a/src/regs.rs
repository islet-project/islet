#![allow(unused_attributes)]

#[macro_use]
mod macros;

mod cptr_el2;
mod id_aa64pfr1_el1;
mod smcr_el2;
mod zcr_el1;
mod zcr_el2;

pub use cptr_el2::CPTR_EL2;
pub use id_aa64pfr1_el1::ID_AA64PFR1_SME_EL1;
pub use smcr_el2::SMCR_EL2;
pub use zcr_el1::ZCR_EL1;
pub use zcr_el2::ZCR_EL2;

use crate::bits_in_reg;

define_bits!(
    EsrEl1,
    // Exception Class.
    EC[31 - 26],
    // Instruction Length for synchronous exceptions.
    IL[25 - 25],
    // Syndrome information.
    ISS[24 - 0]
);

define_bits!(
    EsrEl2,
    // Exception Class.
    EC[31 - 26],
    // Instruction Length for synchronous exceptions.
    IL[25 - 25],
    // Instruction syndrome valid.
    ISV[24 - 24],
    // Syndrome Access Size (ISV == '1')
    SAS[23 - 22],
    // Syndrome Sign Extend (ISV == '1')
    SSE[21 - 21],
    // Syndrome Register Transfer (ISV == '1')
    SRT[20 - 16],
    // Width of the register accessed by the instruction is Sixty-Four (ISV == '1')
    SF[15 - 15],
    // Acquire/Release. (ISV == '1')
    AR[14 - 14],
    // Indicates that the fault came from use of VNCR_EL2 register by EL1 code.
    VNCR[13 - 13],
    // Synchronous Error Type
    SET[12 - 11],
    // FAR not Valid
    FNV[10 - 10],
    // External Abort type
    EA[9 - 9],
    // Cache Maintenance
    CM[8 - 8],
    S1PTW[7 - 7],
    // Write not Read.
    WNR[6 - 6],
    DFSC[5 - 0]
);

impl EsrEl2 {
    pub fn get_access_size_mask(&self) -> u64 {
        match self.get_masked_value(EsrEl2::SAS) {
            0 => 0xff,                // byte
            1 => 0xffff,              // half-word
            2 => 0xffffffff,          // word
            3 => 0xffffffff_ffffffff, // double word
            _ => unreachable!(),      // SAS consists of two bits
        }
    }
}

pub const ESR_EL1_EC_UNKNOWN: u64 = 0;
pub const ESR_EL2_EC_UNKNOWN: u64 = 0;
pub const ESR_EL2_EC_WFX: u64 = 1;
pub const ESR_EL2_EC_FPU: u64 = 7;
pub const ESR_EL2_EC_SVC: u64 = 21;
pub const ESR_EL2_EC_HVC: u64 = 22;
pub const ESR_EL2_EC_SMC: u64 = 23;
pub const ESR_EL2_EC_SYSREG: u64 = 24;
pub const ESR_EL2_EC_SVE: u64 = 25;
pub const ESR_EL2_EC_INST_ABORT: u64 = 32;
pub const ESR_EL2_EC_DATA_ABORT: u64 = 36;
pub const ESR_EL2_EC_SERROR: u64 = 47;

pub const DFSC_PERM_FAULT_MASK: u64 = 0b111100;
pub const DFSC_PERM_FAULTS: u64 = 0b001100; // 0b0011xx

pub const NON_EMULATABLE_ABORT_MASK: u64 =
    EsrEl2::EC | EsrEl2::SET | EsrEl2::FNV | EsrEl2::EA | EsrEl2::DFSC;
pub const EMULATABLE_ABORT_MASK: u64 =
    NON_EMULATABLE_ABORT_MASK | EsrEl2::ISV | EsrEl2::SAS | EsrEl2::SF | EsrEl2::WNR;
pub const INST_ABORT_MASK: u64 = EsrEl2::EC | EsrEl2::SET | EsrEl2::EA | EsrEl2::DFSC;

macro_rules! define_iss_id {
    ($name:ident, $Op0:expr, $Op1:expr, $CRn:expr, $CRm:expr, $Op2:expr) => {
        pub const $name: u32 = bits_in_reg(ISS::Op0, $Op0) as u32
            | bits_in_reg(ISS::Op1, $Op1) as u32
            | bits_in_reg(ISS::CRn, $CRn) as u32
            | bits_in_reg(ISS::CRm, $CRm) as u32
            | bits_in_reg(ISS::Op2, $Op2) as u32;
    };
}

define_bits!(
    ISS,
    IL[25 - 25],
    Op0[21 - 20],
    Op2[19 - 17],
    Op1[16 - 14],
    CRn[13 - 10],
    Rt[9 - 5],
    CRm[4 - 1],
    Direction[0 - 0]
);

define_iss_id!(ISS_ID_AA64PFR0_EL1, 3, 0, 0, 4, 0);

define_iss_id!(ISS_ID_AA64PFR1_EL1, 3, 0, 0, 4, 1);

define_iss_id!(ISS_ID_AA64DFR0_EL1, 3, 0, 0, 5, 0);

define_iss_id!(ISS_ID_AA64DFR1_EL1, 3, 0, 0, 5, 1);

define_iss_id!(ISS_ID_AA64AFR0_EL1, 3, 0, 0, 5, 4);

define_iss_id!(ISS_ID_AA64AFR1_EL1, 3, 0, 0, 5, 5);

define_iss_id!(ISS_ID_AA64ISAR0_EL1, 3, 0, 0, 6, 0);

define_iss_id!(ISS_ID_AA64ISAR1_EL1, 3, 0, 0, 6, 1);

define_iss_id!(ISS_ID_AA64MMFR0_EL1, 3, 0, 0, 7, 0);

define_iss_id!(ISS_ID_AA64MMFR1_EL1, 3, 0, 0, 7, 1);

define_iss_id!(ISS_ID_AA64MMFR2_EL1, 3, 0, 0, 7, 2);
