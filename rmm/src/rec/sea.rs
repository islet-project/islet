use crate::realm::rd::Rd;
use crate::rec::context::{set_reg, RegOffset};
use crate::rec::Rec;
use crate::rmi::error::Error;
use crate::rmi::rec::run::{EntryFlag, Run};

use aarch64_cpu::registers::Readable;
use aarch64_cpu::registers::{HPFAR_EL2, SPSR_EL2};
use armv9a::regs::*;
use armv9a::InMemoryRegister; // re-exported from tock_registers

pub fn host_sea_inject(rec: &mut Rec<'_>, run: &Run) -> Result<(), Error> {
    let flags = run.entry_flags();

    // Host has not completed emulation for an Emulatable Abort.
    if flags.get_masked(EntryFlag::INJECT_SEA) == 0 {
        return Ok(());
    }

    let esr_el2 = rec.context.sys_regs.esr_el2;
    let esr = EsrEl2::new(esr_el2);
    let ec = esr.get_masked_value(EsrEl2::EC);

    if ec != ESR_EL2_EC_DATA_ABORT {
        return Ok(());
    }

    // [Spec] If the most recent exit was due to Data Abort at an Unprotected IPA
    // and enter.flags.inject_sea == RMI_INJECT_SEA,
    // then a Synchronous External Abort is taken to the Realm.
    let fault_ipa = rec.context.sys_regs.hpfar & (HPFAR_EL2::FIPA.mask << HPFAR_EL2::FIPA.shift);
    let fault_ipa = (fault_ipa << 8) as usize;

    let raw_ptr: *const Rd = rec.owner()? as *const Rd;
    let rd: &Rd = unsafe { raw_ptr.as_ref().expect("REASON") }; // FIXME
    if !rd.addr_in_par(fault_ipa) && fault_ipa < rd.ipa_size() {
        debug!("[JBD] about the inject SEA");
        inject_sea(rec, esr_el2, rec.context.sys_regs.far_el2);
    }

    Ok(())
}

pub fn inject_sea(rec: &mut Rec<'_>, esr_el2: u64, far_el2: u64) {
    let mut esr_el1 = esr_el2 & !(EsrEl2::EC | EsrEl2::FNV | EsrEl2::S1PTW | EsrEl2::DFSC);
    let mut ec = esr_el2 & EsrEl2::EC;
    let context = &mut rec.context;
    let spsr_el2: InMemoryRegister<u64, SPSR_EL2::Register> =
        InMemoryRegister::new(context.spsr_el2);
    let elr_el2 = context.elr_el2;
    let spsr_m = spsr_el2.read(SPSR_EL2::M);
    if spsr_m != SPSR_EL2::M::EL0t.into() {
        ec |= 1 << EsrEl2::EC.trailing_zeros();
    }
    esr_el1 |= ec;
    esr_el1 |= EsrEl2::EA;
    esr_el1 |= 0b010000; // Synchronous External Abort (SEA)
    const VBAR_CURRENT_SP0_OFFSET: u64 = 0x0;
    const VBAR_CURRENT_SPX_OFFSET: u64 = 0x200;
    const VBAR_LOWER_AARCH64_OFFSET: u64 = 0x400;
    let mut vector_entry = {
        match spsr_el2.read_as_enum(SPSR_EL2::M) {
            Some(SPSR_EL2::M::Value::EL0t) => VBAR_LOWER_AARCH64_OFFSET,
            Some(SPSR_EL2::M::Value::EL1t) => VBAR_CURRENT_SP0_OFFSET,
            Some(SPSR_EL2::M::Value::EL1h) => VBAR_CURRENT_SPX_OFFSET,
            _ => panic!("shouldn't be reached here"), // Realms run at aarch64 state only (i.e. no aarch32)
        }
    };
    vector_entry += context.sys_regs.vbar;

    let pstate: u64 = (SPSR_EL2::D::SET
        + SPSR_EL2::A::SET
        + SPSR_EL2::I::SET
        + SPSR_EL2::F::SET
        + SPSR_EL2::M::EL1h)
        .into();

    context.sys_regs.esr_el1 = esr_el1;
    context.sys_regs.far = far_el2;
    context.sys_regs.elr = elr_el2;
    context.sys_regs.spsr = spsr_el2.get();
    context.elr_el2 = vector_entry;
    let _ = set_reg(rec, RegOffset::PSTATE, pstate as usize);
}
