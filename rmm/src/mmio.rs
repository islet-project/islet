use crate::rec::{Rec, RmmRecEmulatableAbort::EmulatableAbort};
use crate::rmi::error::Error;
use crate::rmi::rec::run::{Run, REC_ENTRY_FLAG_EMUL_MMIO};
use armv9a::regs::*;

pub fn emulate_mmio(rec: &mut Rec<'_>, run: &Run) -> Result<(), Error> {
    let flags = run.entry_flags();

    // Host has not completed emulation for an Emulatable Abort.
    if (flags & REC_ENTRY_FLAG_EMUL_MMIO) == 0 {
        return Ok(());
    }

    let esr_el2 = rec.context.sys_regs.esr_el2;
    let esr = EsrEl2::new(esr_el2);
    let ec = esr.get_masked_value(EsrEl2::EC);
    let wnr = esr.get_masked_value(EsrEl2::WNR);
    let rt = esr.get_masked_value(EsrEl2::SRT) as usize;

    if ec != ESR_EL2_EC_DATA_ABORT || rec.emulatable_abort() != EmulatableAbort {
        return Err(Error::RmiErrorRec);
    }

    // MMIO read case
    if wnr == 0 && rt != 31 {
        let mask = esr.get_access_size_mask();
        let val = run.entry_gpr(0)? & mask;
        let sign_extended = esr.get_masked_value(EsrEl2::SSE);
        if sign_extended != 0 {
            // TODO
            unimplemented!();
        }
        rec.context.gp_regs[rt] = val;
    }
    rec.context.elr += 4;
    Ok(())
}
