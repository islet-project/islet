use crate::realm::registry::get_realm;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;
use crate::rmi::rec::run::{Run, REC_ENTRY_FLAG_EMUL_MMIO};
use armv9a::regs::*;

pub fn emulate_mmio(id: usize, vcpu: usize, run: &Run) -> Result<(), Error> {
    let realm = get_realm(id).ok_or(Error::RmiErrorOthers(NotExistRealm))?;
    let mut locked_realm = realm.lock();
    let vcpu = locked_realm
        .vcpus
        .get_mut(vcpu)
        .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
    let context = &mut vcpu.lock().context;

    let flags = unsafe { run.entry_flags() };

    // Host has not completed emulation for an Emulatable Abort.
    if (flags & REC_ENTRY_FLAG_EMUL_MMIO) == 0 {
        return Ok(());
    }

    let esr_el2 = context.sys_regs.esr_el2;
    let esr = EsrEl2::new(esr_el2);
    let isv = esr.get_masked_value(EsrEl2::ISV);
    let ec = esr.get_masked_value(EsrEl2::EC);
    let wnr = esr.get_masked_value(EsrEl2::WNR);
    let rt = esr.get_masked_value(EsrEl2::SRT) as usize;

    if ec != ESR_EL2_EC_DATA_ABORT || isv == 0 {
        return Err(Error::RmiErrorRec);
    }

    // MMIO read case
    if wnr == 0 && rt != 31 {
        let mask = esr.get_access_size_mask();
        let val = unsafe { run.entry_gpr(0)? } & mask;
        let sign_extended = esr.get_masked_value(EsrEl2::SSE);
        if sign_extended != 0 {
            // TODO
            unimplemented!();
        }
        context.gp_regs[rt] = val;
    }
    context.elr += 4;
    Ok(())
}
