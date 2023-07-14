use crate::host::Accessor as HostAccessor;

/// The structure holds data passsed between the Host and the RMM
/// on Realm Execution Context (REC) entry and exit.
#[repr(C)]
pub struct Run {
    entry: Entry,
    exit: Exit,
}

impl Run {
    pub unsafe fn entry_flags(&self) -> u64 {
        self.entry.inner.flags.val
    }

    #[allow(dead_code)]
    pub unsafe fn entry_gpr0(&self) -> u64 {
        self.entry.inner.gprs.val[0]
    }

    pub unsafe fn entry_gic_lrs(&self) -> &[u64; 16] {
        &self.entry.inner.gicv3.inner.lrs
    }

    pub unsafe fn entry_gic_hcr(&self) -> u64 {
        self.entry.inner.gicv3.inner.hcr
    }

    pub unsafe fn exit_gic_lrs_mut(&mut self) -> &mut [u64; 16] {
        &mut (*(*self.exit.inner).gicv3.inner).lrs
    }

    pub unsafe fn set_imm(&mut self, imm: u16) {
        (*self.exit.inner).imm.val = imm;
    }

    pub unsafe fn set_exit_reason(&mut self, exit_reason: u8) {
        (*self.exit.inner).exit_reason.val = exit_reason;
    }

    pub unsafe fn set_esr(&mut self, esr: u64) {
        (*(*self.exit.inner).sys_regs.inner).esr = esr;
    }

    pub unsafe fn set_far(&mut self, far: u64) {
        (*(*self.exit.inner).sys_regs.inner).far = far;
    }

    pub unsafe fn set_hpfar(&mut self, hpfar: u64) {
        (*(*self.exit.inner).sys_regs.inner).hpfar = hpfar;
    }

    pub unsafe fn set_ripas(&mut self, base: u64, size: u64, state: u8) {
        (*(*self.exit.inner).ripas.inner).base = base;
        (*(*self.exit.inner).ripas.inner).size = size;
        (*(*self.exit.inner).ripas.inner).value = state;
    }

    pub unsafe fn set_gic_lrs(&mut self, src: &[u64], len: usize) {
        (*(*self.exit.inner).gicv3.inner).lrs[..len].copy_from_slice(&src[..len])
    }

    pub unsafe fn set_gic_misr(&mut self, val: u64) {
        (*(*self.exit.inner).gicv3.inner).misr = val;
    }

    pub unsafe fn set_gic_vmcr(&mut self, val: u64) {
        (*(*self.exit.inner).gicv3.inner).vmcr = val;
    }

    pub unsafe fn set_gic_hcr(&mut self, val: u64) {
        (*(*self.exit.inner).gicv3.inner).hcr = val;
    }
}

impl core::fmt::Debug for Run {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Safety: union type should be initialized
        unsafe {
            f.debug_struct("rec::Run")
                .field(
                    "entry::flags",
                    &format_args!("{:#X}", &self.entry.inner.flags.val),
                )
                .field("entry::gprs", &self.entry.inner.gprs.val)
                .field(
                    "entry::gicv3_hcr",
                    &format_args!("{:#X}", &self.entry.inner.gicv3.inner.hcr),
                )
                .field("entry::gicv3_lrs", &self.entry.inner.gicv3.inner.lrs)
                .field("exit::exit_reason", &self.exit.inner.exit_reason.val)
                .field("exit::imm", &self.exit.inner.imm.val)
                .finish()
        }
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        // TODO: recursive drop
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.entry.inner);
            core::mem::ManuallyDrop::drop(&mut self.exit.inner);
        }
    }
}

/// The structure holds data passsed from the Host to the RMM on REC entry.
#[repr(C)]
union Entry {
    inner: core::mem::ManuallyDrop<EntryInner>,
    reserved: [u8; 0x800],
}

/// The structure holds data passsed from the RMM to the Host on REC exit.
#[repr(C)]
union Exit {
    inner: core::mem::ManuallyDrop<ExitInner>,
    reserved: [u8; 0x1000 - 0x800],
}

#[repr(C)]
struct EntryInner {
    /// Flags
    flags: Flags,
    /// General-purpose registers
    gprs: GPRs,
    /// Generic Interrupt Controller version 3
    gicv3: EntryGICv3,
}

#[repr(C)]
union Flags {
    val: u64,
    reserved: [u8; 0x200],
}

/// Whether the host has completed emulation for an Emulatable Data Abort.
///  val 0: Host has not completed emulation for an Emulatable Abort.
///  val 1: Host has completed emulation for an Emulatable Abort.
pub const REC_ENTRY_FLAG_EMUL_MMIO: u64 = 1 << 0;
/// Whether to inject a Synchronous External Abort (SEA) into the Realm.
///  val 0: Do not inject an SEA into the Realm.
///  val 1: Inject an SEA into the Realm.
#[allow(dead_code)]
pub const REC_ENTRY_FLAG_INJECT_SEA: u64 = 1 << 1;
/// Whether to trap WFI execution by the Realm.
///  val 0: Trap is disabled.
///  val 1: Trap is enabled.
#[allow(dead_code)]
pub const REC_ENTRY_FLAG_TRAP_WFI: u64 = 1 << 2;
/// Whether to trap WFE execution by the Realm.
///  val 0: Trap is disabled.
///  val 1: Trap is enabled.
#[allow(dead_code)]
pub const REC_ENTRY_FLAG_TRAP_WFE: u64 = 1 << 3;

/// General-purpose registers
#[repr(C)]
union GPRs {
    val: [u64; 31],
    reserved: [u8; 0x300 - 0x200],
}

/// Generic Interrupt Controller version 3
#[repr(C)]
union EntryGICv3 {
    inner: core::mem::ManuallyDrop<EntryGICv3Inner>,
    reserved: [u8; 0x800 - 0x300],
}

#[repr(C)]
struct EntryGICv3Inner {
    /// Hypervisor Control Register
    hcr: u64,
    /// List Registers
    lrs: [u64; 16],
}

#[repr(C)]
struct ExitInner {
    exit_reason: ExitReason,
    sys_regs: SysRegs,
    gprs: GPRs,
    gicv3: ExitGICv3,
    cnt: CounterTimer,
    ripas: RIPAS,
    imm: Imm,
}

#[repr(C)]
union ExitReason {
    val: u8,
    reserved: [u8; 0x100],
}

#[repr(C)]
union SysRegs {
    inner: core::mem::ManuallyDrop<SysRegsInner>,
    reserved: [u8; 0x200 - 0x100],
}

#[repr(C)]
struct SysRegsInner {
    /// Exception Syndrome Register
    esr: u64,
    /// Fault Address Register
    far: u64,
    /// Hypervisor IPA Fault Address Register
    hpfar: u64,
}

/// Generic Interrupt Controller version 3
#[repr(C)]
union ExitGICv3 {
    inner: core::mem::ManuallyDrop<ExitGICv3Inner>,
    reserved: [u8; 0x400 - 0x300],
}

#[repr(C)]
struct ExitGICv3Inner {
    /// Hypervisor Control Register
    hcr: u64,
    /// List Registers
    lrs: [u64; 16],
    /// Maintenance Interrupt State Register
    misr: u64,
    /// Virtual Machine Control Register
    vmcr: u64,
}

#[repr(C)]
union CounterTimer {
    inner: core::mem::ManuallyDrop<CounterTimerInner>,
    reserved: [u8; 0x500 - 0x400],
}

#[repr(C)]
struct CounterTimerInner {
    /// Physical Timer Control Register
    p_ctl: u64,
    /// Physical Timer CompareValue Register
    p_cval: [u64; 16],
    /// Virtual Timer Control Register
    v_ctl: u64,
    /// Virtual Timer CompareValue Register
    v_cval: u64,
}

/// Realm IPA (Intermediate Physical Address) State
#[repr(C)]
union RIPAS {
    inner: core::mem::ManuallyDrop<RIPASInner>,
    reserved: [u8; 0x600 - 0x500],
}

#[repr(C)]
struct RIPASInner {
    base: u64,
    size: u64,
    value: u8,
}

/// Host call immediate value
#[repr(C)]
union Imm {
    val: u16,
    reserved: [u8; 0x800 - 0x600],
}

impl HostAccessor for Run {}
