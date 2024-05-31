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

define_sys_register!(
    ESR_EL2,
    // Exception Class.
    EC[31 - 26],
    // Instruction Length for synchronous exceptions.
    IL[25 - 25],
    // Instruction Specific Syndrome.
    ISS[24 - 0],
    ISS_BRK_CMT[15 - 0],
    S1PTW[7 - 7],
    DFSC[5 - 0]
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

pub const NON_EMULATABLE_ABORT_MASK: u64 =
    EsrEl2::EC | EsrEl2::SET | EsrEl2::FNV | EsrEl2::EA | EsrEl2::DFSC;
pub const EMULATABLE_ABORT_MASK: u64 =
    NON_EMULATABLE_ABORT_MASK | EsrEl2::ISV | EsrEl2::SAS | EsrEl2::SF | EsrEl2::WNR;

define_sys_register!(
    VTCR_EL2, // ref. Virtualzation Translation Control Register
    DS[32 - 32],
    RES1[31 - 31],
    // Non-secure stage 2 translation output address space for the Secure EL1&0
    // translation regime
    // 0b0: All stage 2 translations for the Non-secure IPA space of the Secure EL1&0
    //      translation regime acccess the Secure PA space
    // 0b1: All stage 2 translations for the Non-secure IPA space of the secure EL1&0
    //      trnaslation regmime access the non-secure PA space
    NSA[30 - 30],
    // Non-secure stage 2 translation table address space for the Secure EL1&0
    // translation regime
    NSW[29 - 29],
    HWU62[28 - 28],
    HWU61[27 - 27],
    HWU60[26 - 26],
    HWU59[25 - 25],
    RES0[24 - 23],
    HD[22 - 22],
    HA[21 - 21],
    VS[19 - 19],    // VMID size. 0b0: 8bits, 0b1: 16bit
    PS[18 - 16],    // Physical address size for the second stage of translation
    TG0[15 - 14],   // Granule size (VTTBR_EL2)
    SH0[13 - 12],   // Shareability (VTTBR_EL2)
    ORGN0[11 - 10], // Outer cacheability (VTTBR_EL2)
    IRGN0[9 - 8],   // Outer cacheability (VTTBR_EL2)
    SL0[7 - 6],     // Starting level of the stage 2 translation lookup
    T0SZ[5 - 0]     // Size offset of the memory region (TTBR0_EL2)
);

pub mod vtcr_sl0 {
    pub const SL0_4K_L2: u64 = 0x0;
    pub const SL0_4K_L1: u64 = 0x1;
    pub const SL0_4K_L0: u64 = 0x2;
    pub const SL0_4K_L3: u64 = 0x3;
}

pub mod tcr_paddr_size {
    // PS
    pub const PS_4G: u64 = 0b000; // 32bits
    pub const PS_64G: u64 = 0b001; // 36bits
    pub const PS_1T: u64 = 0b010; // 40bits
    pub const PS_4T: u64 = 0b011; // 42bits
    pub const PS_16T: u64 = 0b100; // 44bits
    pub const PS_256T: u64 = 0b101; // 48bits
    pub const PS_4P: u64 = 0b110; // 52bits
}

pub mod tcr_granule {
    // TG0
    pub const G_4K: u64 = 0b00;
    pub const G_64K: u64 = 0b01;
    pub const G_16K: u64 = 0b10;
}

pub mod tcr_shareable {
    // SH0
    pub const NONE: u64 = 0b00;
    pub const OUTER: u64 = 0b10;
    pub const INNER: u64 = 0b11;
}

pub mod tcr_cacheable {
    // ORGN0, IRGN0
    pub const NONE: u64 = 0b00; // NonCacheable
    pub const WBWA: u64 = 0b01; // Write-Back; Read-Alloc; Write-Alloc
    pub const WTNWA: u64 = 0b10; // Write-thru; Read-Alloc; No Write-Alloc
    pub const WBNWA: u64 = 0b11; // Write-Back; Read-Alloc; No Write-Alloc
}

pub mod tcr_start_level {
    // SL0
    pub const L2: u64 = 0b00;
    pub const L1: u64 = 0b01;
    pub const L0: u64 = 0b10;
}

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
