use crate::const_assert_eq;
use crate::granule::GRANULE_SIZE;
use crate::host::Accessor as HostAccessor;
use crate::rmi::error::Error;

use autopadding::*;

/// The structure holds data passsed between the Host and the RMM
/// on Realm Execution Context (REC) entry and exit.
#[repr(C)]
#[derive(Default)]
pub struct Run {
    entry: Entry,
    exit: Exit,
}
const_assert_eq!(core::mem::size_of::<Run>(), GRANULE_SIZE);

pad_struct_and_impl_default!(
struct Entry {
    0x0   flags: u64,
    0x200 gprs: [u64; NR_GPRS],
    0x300 gicv3_hcr: u64,
    0x308 gicv3_lrs: [u64; NR_GIC_LRS],
    0x800 => @END,
}
);

pad_struct_and_impl_default!(
struct Exit {
    0x0   exit_reason: u8,
    0x100 esr: u64,
    0x108 far: u64,
    0x110 hpfar: u64,
    0x200 gprs: [u64; NR_GPRS],
    0x300 gicv3_hcr: u64,
    0x308 gicv3_lrs: [u64; NR_GIC_LRS],
    0x388 gicv3_misr: u64,
    0x390 gicv3_vmcr: u64,
    0x400 cntp_ctl: u64,
    0x408 cntp_cval: u64,
    0x410 cntv_ctl: u64,
    0x418 cntv_cval: u64,
    0x500 ripas_base: u64,
    0x508 ripas_size: u64,
    0x510 ripas_value: u8,
    0x600 imm: u16,
    0x700 pmu_ovf: u64,
    0x708 pmu_intr_en: u64,
    0x710 pmu_cntr_en: u64,
    0x800 => @END,
}
);

impl Run {
    pub fn entry_flags(&self) -> u64 {
        self.entry.flags
    }

    pub fn entry_gpr(&self, idx: usize) -> Result<u64, Error> {
        if idx >= NR_GPRS {
            error!("out of index: {}", idx);
            return Err(Error::RmiErrorInput);
        }
        Ok(self.entry.gprs[idx])
    }

    pub fn entry_gic_lrs(&self) -> &[u64; 16] {
        &self.entry.gicv3_lrs
    }

    pub fn entry_gic_hcr(&self) -> u64 {
        self.entry.gicv3_hcr
    }

    pub fn exit_gic_lrs_mut(&mut self) -> &mut [u64; 16] {
        &mut self.exit.gicv3_lrs
    }

    pub fn set_imm(&mut self, imm: u16) {
        self.exit.imm = imm;
    }

    pub fn set_exit_reason(&mut self, exit_reason: u8) {
        self.exit.exit_reason = exit_reason;
    }

    pub fn set_esr(&mut self, esr: u64) {
        self.exit.esr = esr;
    }

    pub fn set_far(&mut self, far: u64) {
        self.exit.far = far;
    }

    pub fn set_hpfar(&mut self, hpfar: u64) {
        self.exit.hpfar = hpfar;
    }

    pub fn set_gpr(&mut self, idx: usize, val: u64) -> Result<(), Error> {
        if idx >= NR_GPRS {
            error!("out of index: {}", idx);
            return Err(Error::RmiErrorInput);
        }
        self.exit.gprs[idx] = val;
        Ok(())
    }

    pub fn set_ripas(&mut self, base: u64, size: u64, state: u8) {
        self.exit.ripas_base = base;
        self.exit.ripas_size = size;
        self.exit.ripas_value = state;
    }

    pub fn set_gic_lrs(&mut self, src: &[u64], len: usize) {
        self.exit.gicv3_lrs.copy_from_slice(&src[..len])
    }

    pub fn set_gic_misr(&mut self, val: u64) {
        self.exit.gicv3_misr = val;
    }

    pub fn set_gic_vmcr(&mut self, val: u64) {
        self.exit.gicv3_vmcr = val;
    }

    pub fn set_gic_hcr(&mut self, val: u64) {
        self.exit.gicv3_hcr = val;
    }

    pub fn set_cntv_ctl(&mut self, val: u64) {
        self.exit.cntv_ctl = val;
    }

    pub fn set_cntv_cval(&mut self, val: u64) {
        self.exit.cntv_cval = val;
    }

    pub fn set_cntp_ctl(&mut self, val: u64) {
        self.exit.cntp_ctl = val;
    }

    pub fn set_cntp_cval(&mut self, val: u64) {
        self.exit.cntp_cval = val;
    }
}

impl core::fmt::Debug for Run {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("rec::Run")
            .field("entry::flags", &format_args!("{:#X}", &self.entry.flags))
            .field("entry::gprs", &self.entry.gprs)
            .field(
                "entry::gicv3_hcr",
                &format_args!("{:#X}", &self.entry.gicv3_hcr),
            )
            .field("entry::gicv3_lrs", &self.entry.gicv3_lrs)
            .field("exit::exit_reason", &self.exit.exit_reason)
            .field("exit::imm", &self.exit.imm)
            .field("exit::cntp_ctl", &self.exit.cntp_ctl)
            .field("exit::cntp_cval", &self.exit.cntp_cval)
            .field("exit::cntv_ctl", &self.exit.cntv_ctl)
            .field("exit::cntv_cval", &self.exit.cntv_cval)
            .finish()
    }
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
pub const NR_GPRS: usize = 31;
const NR_GIC_LRS: usize = 16;

impl HostAccessor for Run {
    fn validate(&self) -> bool {
        const ICH_LR_HW_OFFSET: usize = 61;
        // A6.1 Realm interrupts, HW == '0'
        for lr in &self.entry.gicv3_lrs {
            if lr & (1 << ICH_LR_HW_OFFSET) != 0 {
                return false;
            }
        }
        true
    }
}
