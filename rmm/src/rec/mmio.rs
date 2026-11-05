use crate::rec::{Rec, RmmRecEmulatableAbort::EmulatableAbort};
use crate::rmi::error::Error;
use crate::rmi::rec::run::{EntryFlag, Run};
use armv9a::regs::*;

/// Sign-extends a value of the access size (SAS) to the destination register width.
/// If `sf` is true, the destination is Xt (64-bit). Otherwise it is Wt (32-bit),
/// and the upper 32 bits are zeroed.
fn sign_extend(val: u64, sas: u64, sf: bool) -> u64 {
    let bits = 8u64 << sas; // SAS: 0=byte, 1=half, 2=word, 3=dword
    let shift = 64 - bits;
    let extended = (((val << shift) as i64) >> shift) as u64;
    if sf {
        extended
    } else {
        extended & 0xffff_ffff
    }
}

pub fn emulate_mmio(rec: &mut Rec<'_>, run: &Run) -> Result<(), Error> {
    let flags = run.entry_flags();

    // Host has not completed emulation for an Emulatable Abort.
    // if INJECT_SEA is set then the value of EMUL_MMIO is ignored.
    if flags.get_masked(EntryFlag::INJECT_SEA | EntryFlag::EMUL_MMIO) == 0 {
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
        let mut val = run.entry_gpr(0)? & mask;
        let sign_extended = esr.get_masked_value(EsrEl2::SSE);
        if sign_extended != 0 {
            let sas = esr.get_masked_value(EsrEl2::SAS);
            let sf = esr.get_masked_value(EsrEl2::SF) != 0;
            val = sign_extend(val, sas, sf);
        }
        rec.context.gp_regs[rt] = val;
    }
    rec.context.elr_el2 += 4;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::sign_extend;

    #[test]
    fn sign_extend_byte_to_w_and_x() {
        assert_eq!(sign_extend(0x80, 0, false), 0xffff_ff80);
        assert_eq!(sign_extend(0x80, 0, true), 0xffff_ffff_ffff_ff80);
        assert_eq!(sign_extend(0x7f, 0, false), 0x7f);
    }

    #[test]
    fn sign_extend_word_to_x() {
        assert_eq!(sign_extend(0x8000_0000, 2, true), 0xffff_ffff_8000_0000);
    }

    #[test]
    fn sign_extend_dword_is_identity() {
        assert_eq!(sign_extend(0x8000_0000_0000_0000, 3, true), 0x8000_0000_0000_0000);
    }
}
