define_sys_register!(
    MPIDR_EL1,     // ref. D7.2.74
    AFF2[23 - 16], // Affinity level 2
    AFF1[15 - 08]  // Affinity level 1
);

define_sys_register!(CurrentEL, EL[3 - 2]);
pub fn current_el() -> u8 {
    unsafe { CurrentEL.get_masked_value(CurrentEL::EL) as u8 }
}

define_sys_register!(VBAR_EL2, RES0[10 - 0]);

define_sys_register!(
    ESR_EL2,
    // Exception Class.
    EC[31 - 26],
    // Instruction Length for synchronous exceptions.
    IL[25 - 25],
    // Instruction Specific Syndrome.
    ISS[24 - 00],
    ISS_BRK_CMT[15 - 00],
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
    S1PTW[7 - 7],
    // Write not Read.
    WNR[6 - 6],
    DFSC[5 - 0]
);

pub const ESR_EL2_EC_DATA_ABORT: u64 = 36;

define_register!(SP);
define_sys_register!(SP_EL0);
define_sys_register!(SP_EL1);
define_sys_register!(SP_EL2);

define_sys_register!(
    SPSR_EL2,
    D[9 - 9], // Debug exception mask
    A[8 - 8], // SError exception mask
    I[7 - 7], // IRQ exception mask
    F[6 - 6], // FIQ exception mask
    M[3 - 0]  // Exception level and selected SP
);

define_sys_register!(ELR_EL2);

define_sys_register!(TPIDR_EL2);
use crate::realm::context::Context;
use monitor::realm::vcpu::VCPU;
pub unsafe fn current_vcpu() -> Option<&'static mut VCPU<Context>> {
    match TPIDR_EL2.get() {
        0 => None,
        current => Some(&mut *(current as *mut VCPU<Context>)),
    }
}

define_sys_register!(
    HCR_EL2, // ref. D13.2.46\
    FWB[46 - 46],
    TEA[37 - 37],
    TERR[36 - 36],
    TLOR[35 - 35],
    E2H[34 - 34],
    ID[33 - 33],    // Disables stage 2 instruction cache
    CD[32 - 32],    // Disables stage 2 data cache
    RW[31 - 31],    // Execution state control for lower Exception level
    TRVM[30 - 30],  // Trap reads of Virtual Memory controls
    TDZ[28 - 28],   // Traps DC ZVA instruction
    TGE[27 - 27],   // Traps general exceptions
    TVM[26 - 26],   // Traps virtual memory controls
    TTLB[25 - 25],  // Traps TLB maintenance instructions
    TPU[24 - 24],   // Traps cache maintenance instructions to Point of Unification (POU)
    TPC[23 - 23], // Traps data or unified cache maintenance instructions to Point of Coherency (POC)
    TSW[22 - 22], // Traps data or unified cache maintenance instructions by Set or Way
    TACR[21 - 21], // Traps Auxiliary Control registers
    TIDCP[20 - 20], // Traps Implementation Dependent functionality
    TSC[19 - 19], // Traps SMC instruction.
    TID3[18 - 18], // Traps ID group 3
    TID2[17 - 17], // Traps ID group 2
    TID1[16 - 16], // Traps ID group 1
    TID0[15 - 15], // Traps ID group 0
    TWE[14 - 14], // Traps WFE instruction
    TWI[13 - 13], // Traps WFI instruction
    DC[12 - 12],  // Default cacheable
    BSU[11 - 10], // Barrier shareability upgrade
    FB[09 - 09],  // Forces broadcast
    VSE[08 - 08], // Virtual System Error/Asynchronous Abort.
    VI[07 - 07],  // Virtual IRQ interrupt
    VF[06 - 06],  // Virtual FRQ interrupt
    AMO[05 - 05], // Asynchronous abort and error interrupt routing
    IMO[04 - 04], // Physical IRQ routing
    FMO[03 - 03], // Physical FIQ routing
    PTW[02 - 02], // Protected Table Walk
    SWIO[01 - 01],
    VM[00 - 00],             // Virtualization enable
    RES0[63 - 34 | 29 - 29]  //RES1[01 - 01]
);

define_sys_register!(
    SCTLR_EL2,
    EE[25 - 25],  // Endianness of data accesses at EL2
    WXN[19 - 19], // Write permission implies Execute-never
    I[12 - 12],   // Instruction access Cacheability at EL2
    EOS[11 - 11], // Exception exit is a context synchronization event
    SA[3 - 3],    // SP Alignment check enable
    C[2 - 2],     // Data access Cacheability  at EL2
    A[1 - 1],     // Alignment check enable
    M[0 - 0]      // MMU enable for EL2
);

define_sys_register!(
    ID_AA64MMFR0_EL1, // ref. D7.2.43: AArch64 Memory Model Feature Register 0
    TGran4[31 - 28],
    TGran64[27 - 24],
    TGran16[23 - 20],
    BigEndEL0[19 - 16],
    SNSMem[15 - 12],
    BigEnd[11 - 08],
    ASIDBits[07 - 04],
    PARange[03 - 00]
);

define_sys_register!(
    MAIR_EL2, // ref. D7.2.71: Memory Attribute Indirection Register
    Attr7[63 - 56],
    Attr6[55 - 48],
    Attr5[47 - 40],
    Attr4[39 - 32],
    Attr3[31 - 24],
    Attr2[23 - 16],
    Attr1[15 - 8],
    Attr0[7 - 0]
);

pub mod mair_attr {
    // N: non
    // G: Gathering, R: Reodering, E: Early write-back
    pub const DEVICE_NGNRNE: u64 = 0b0000; // 0x0
    pub const DEVICE_NGNRE: u64 = 0b0100; // 0x4
    pub const DEVICE_GRE: u64 = 0b1100; // 0xc
    pub const NORMAL_NC: u64 = 0b0100_0100; // 0x44, normal memory, non-cacheable
    pub const NORMAL: u64 = 0b1111_1111; // 0xff, nomral memory, inner read-alloc, write-alloc,wb, non-transient
}

define_sys_register!(
    TTBR0_EL2, // ref. Translation Table Base Register 0(EL2)
    ASID[63 - 48],
    BADDR[47 - 1],
    CNP[0 - 0]
);

define_sys_register!(
    TCR_EL2, // ref. Translation Control Register (EL2)
    MTX[33 - 33],
    DS[32 - 32],
    TCMA[30 - 30],
    TBID[29 - 29],
    HPD[24 - 24],
    HD[22 - 22],
    HA[21 - 21],
    TBI[20 - 20],
    PS[18 - 16],
    TG0[15 - 14],
    SH0[13 - 12],
    ORGN0[11 - 10],
    IRGN0[9 - 8],
    T0SZ[5 - 0]
);

define_sys_register!(
    VTCR_EL2,
    VS[19 - 19],    // VMID size. 0b0: 8bits, 0b1: 16bit
    PS[18 - 16],    // Physical address size for the second stage of translation
    TG0[15 - 14],   // Granule size (VTTBR_EL2)
    SH0[13 - 12],   // Shareability (VTTBR_EL2)
    ORGN0[11 - 10], // Outer cacheability (VTTBR_EL2)
    IRGN0[9 - 8],   // Outer cacheability (VTTBR_EL2)
    SL0[7 - 6],     // Starting level of the stage 2 translation lookup
    T0SZ[5 - 0]     // Size offset of the memory region (TTBR0_EL2)
);

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

define_sys_register!(
    VTTBR_EL2,
    VMID[63 - 48], // The VMID for the translation table
    BADDR[47 - 0]  // Translation table base address
);

define_sys_register!(
    HPFAR_EL2, // Ref. D13.2.55
    NS[63 - 63],
    FIPA[43 - 4] //
);

define_sys_register!(
    FAR_EL2, // Ref. D13.2.55
    OFFSET[11 - 0]
);

define_sys_register!(CPTR_EL2, TAM[30 - 30]);

// GIC-related
define_sys_register!(
    ICH_VTR_EL2,  // Ref. Interrupt Controller VGIC Type Register
    PRI[31 - 29], // The number of virtual priority bits implemented, minus one.
    PRE[28 - 26], // The number of virtual preemption bits implemented, minus one.
    ID[25 - 23], // The number of virtual interrupt identifier bits supported (0b000 means 16 bits while 0b001 means 24 bits)
    LIST[4 - 0]  // The number of implemented List registers, minus one
);

define_sys_register!(ICH_LR0_EL2);
define_sys_register!(ICH_LR1_EL2);
define_sys_register!(ICH_LR2_EL2);
define_sys_register!(ICH_LR3_EL2);
define_sys_register!(ICH_LR4_EL2);
define_sys_register!(ICH_LR5_EL2);
define_sys_register!(ICH_LR6_EL2);
define_sys_register!(ICH_LR7_EL2);
define_sys_register!(ICH_LR8_EL2);
define_sys_register!(ICH_LR9_EL2);
define_sys_register!(ICH_LR10_EL2);
define_sys_register!(ICH_LR11_EL2);
define_sys_register!(ICH_LR12_EL2);
define_sys_register!(ICH_LR13_EL2);
define_sys_register!(ICH_LR14_EL2);
define_sys_register!(ICH_LR15_EL2);

define_sys_register!(ICH_AP0R0_EL2);
define_sys_register!(ICH_AP0R1_EL2);
define_sys_register!(ICH_AP0R2_EL2);
define_sys_register!(ICH_AP0R3_EL2);
define_sys_register!(ICH_AP1R0_EL2);
define_sys_register!(ICH_AP1R1_EL2);
define_sys_register!(ICH_AP1R2_EL2);
define_sys_register!(ICH_AP1R3_EL2);

define_sys_register!(ICH_VMCR_EL2);
define_sys_register!(ICH_HCR_EL2);
define_sys_register!(ICH_MISR_EL2);

define_sys_register!(
    ICC_SRE_EL2,
    ENABLE[3 - 3],
    DIB[2 - 2],
    DFB[1 - 1],
    SRE[0 - 0]
);
define_sys_register!(CNTHCTL_EL2, EL1PCTEN[11 - 11], EL1PTEN[10 - 10]);
