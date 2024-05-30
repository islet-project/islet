mod frame;
pub mod syndrome;

use self::frame::TrapFrame;
use self::syndrome::Fault;
use self::syndrome::Syndrome;
use super::lower::synchronous;
use crate::cpu;
use crate::event::realmexit::{ExitSyncType, RecExitReason};
use crate::mm::translation::PageTable;
use crate::rec::Rec;

use aarch64_cpu::registers::*;

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
pub enum Source {
    CurrentSPEL0,
    CurrentSPELx,
    LowerAArch64,
    LowerAArch32,
}

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
pub enum Kind {
    Synchronous,
    Irq,
    Fiq,
    SError,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Info {
    source: Source,
    kind: Kind,
}

/// This function is called when an exception occurs from CurrentSPEL0, CurrentSPELx.
/// The `info` parameter specifies source (first 16 bits) and kind (following 16
/// bits) of the exception.
/// The `esr` has the value of a syndrome register (ESR_ELx) holding the cause
/// of the Synchronous and SError exception.
/// The `tf` has the TrapFrame of current context.
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn handle_exception(info: Info, esr: u32, tf: &mut TrapFrame) {
    match info.kind {
        Kind::Synchronous => match Syndrome::from(esr) {
            Syndrome::Brk(b) => {
                debug!("brk #{}", b);
                debug!("{:?}\nESR: {:X}\n{:#X?}", info, esr, tf);
                tf.elr += 4; //continue
            }
            Syndrome::PCAlignmentFault => {
                debug!("PCAlignmentFault");
            }
            Syndrome::DataAbort(fault) => {
                let far = FAR_EL2.get();
                debug!("Data Abort (higher), far:{:X}", far);
                match fault {
                    Fault::AddressSize { level } => {
                        debug!("address size, level:{}", level);
                    }
                    Fault::Translation { level } => {
                        debug!("translation, level:{}, esr:{:X}", level, esr);
                        PageTable::get_ref().map(far as usize, true);
                    }
                    Fault::AccessFlag { level } => {
                        debug!("access flag, level:{}", level);
                    }
                    Fault::Permission { level } => {
                        debug!("permission, level:{}", level);
                    }
                    Fault::Alignment => {
                        debug!("alignment");
                    }
                    Fault::TLBConflict => {
                        debug!("tlb conflict");
                    }
                    Fault::Other(_x) => {
                        debug!("other");
                    }
                }
            }
            Syndrome::InstructionAbort(v) => {
                debug!("Instruction Abort (higher)");
            }
            Syndrome::HVC => {
                debug!("HVC");
            }
            Syndrome::SMC => {
                debug!("SMC");
            }
            Syndrome::SysRegInst => {
                debug!("SysRegInst");
            }
            Syndrome::WFX => {
                debug!("WFX");
            }
            Syndrome::Other(v) => {
                debug!("Other");
            }
            undefined => {
                panic!(
                    "{:?} and esr {:x}, TrapFrame: {:?} on cpu::id {:?}",
                    info,
                    esr,
                    tf,
                    cpu::id()
                );
            }
        },
        _ => {
            panic!(
                "Unknown exception! Info={:?}, ESR={:x} on CPU {:?}",
                info,
                esr,
                cpu::id()
            );
        }
    }
}

pub const RET_TO_REC: u64 = 0;
pub const RET_TO_RMM: u64 = 1;
/// This function is called when an exception occurs from LowerAArch64.
/// To enter RMM (EL2), return 1. Otherwise, return 0 to go back to EL1.
/// The `info` parameter specifies source (first 16 bits) and kind (following 16
/// bits) of the exception.
/// The `esr` has the value of a syndrome register (ESR_ELx) holding the cause
/// of the Synchronous and SError exception.
/// The `rec` has the Rec context.
/// The `tf` has the TrapFrame of current context.
///
/// Do not write sys_regs of Rec here. (ref. HANDLE_LOWER in vectors.s)
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn handle_lower_exception(
    info: Info,
    esr: u32,
    rec: &mut Rec<'_>,
    tf: &mut TrapFrame,
) -> u64 {
    match info.kind {
        // TODO: adjust elr according to the decision that kvm made
        Kind::Synchronous => match Syndrome::from(esr) {
            Syndrome::HVC => {
                debug!("Synchronous: HVC: {:#X}", rec.context.gp_regs[0]);

                // Inject undefined exception to the realm
                SPSR_EL1.set(rec.context.spsr);
                ELR_EL1.set(rec.context.elr);
                ESR_EL1.write(ESR_EL1::EC::Unknown + ESR_EL1::IL::SET);

                // Return to realm's exception handler
                let vbar = rec.context.sys_regs.vbar;
                const SPSR_EL2_MODE_EL1H_OFFSET: u64 = 0x200;
                rec.context.elr = vbar + SPSR_EL2_MODE_EL1H_OFFSET;

                tf.regs[0] = RecExitReason::Sync(ExitSyncType::Undefined).into();
                tf.regs[1] = esr as u64;
                tf.regs[2] = 0;
                tf.regs[3] = FAR_EL2.get();
                RET_TO_REC
            }
            Syndrome::SMC => {
                tf.regs[0] = RecExitReason::Sync(ExitSyncType::RSI).into();
                tf.regs[1] = rec.context.gp_regs[0]; // RSI command
                advance_pc(rec);
                RET_TO_RMM
            }
            Syndrome::InstructionAbort(_) | Syndrome::DataAbort(_) => {
                debug!("Synchronous: InstructionAbort | DataAbort");
                if let Syndrome::InstructionAbort(_) = Syndrome::from(esr) {
                    tf.regs[0] = RecExitReason::Sync(ExitSyncType::InstAbort).into()
                } else {
                    tf.regs[0] = RecExitReason::Sync(ExitSyncType::DataAbort).into();
                }
                tf.regs[1] = esr as u64;
                tf.regs[2] = HPFAR_EL2.get();
                tf.regs[3] = FAR_EL2.get();
                let fipa = HPFAR_EL2.read(HPFAR_EL2::FIPA) << 8;
                debug!("fipa: {:X}", fipa);
                debug!("esr_el2: {:X}", esr);
                RET_TO_RMM
            }
            Syndrome::SysRegInst => {
                debug!("Synchronous: MRS, MSR System Register Instruction");
                let ret = synchronous::sys_reg::handle(rec, esr as u64);
                advance_pc(rec);
                ret
            }
            Syndrome::WFX => {
                debug!("Synchronous: WFx");
                tf.regs[0] = RecExitReason::Sync(ExitSyncType::Undefined).into();
                tf.regs[1] = esr as u64;
                tf.regs[2] = HPFAR_EL2.get();
                tf.regs[3] = FAR_EL2.get();
                advance_pc(rec);
                RET_TO_RMM
            }
            undefined => {
                debug!("Synchronous: Other");
                tf.regs[0] = RecExitReason::Sync(ExitSyncType::Undefined).into();
                tf.regs[1] = esr as u64;
                tf.regs[2] = HPFAR_EL2.get();
                tf.regs[3] = FAR_EL2.get();
                RET_TO_RMM
            }
        },
        Kind::Irq => {
            debug!("IRQ");
            tf.regs[0] = RecExitReason::IRQ.into();
            // IRQ isn't interpreted with esr. It just hold previsou info. Void them out.
            tf.regs[1] = 0;
            tf.regs[2] = 0;
            tf.regs[3] = 0;
            RET_TO_RMM
        }
        _ => {
            error!(
                "Unknown exception! Info={:?}, ESR={:x} on CPU {:?}",
                info,
                esr,
                cpu::id()
            );
            RET_TO_REC
        }
    }
}

#[inline(always)]
fn advance_pc(rec: &mut Rec<'_>) {
    rec.context.elr += 4;
}
