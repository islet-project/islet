mod frame;
mod syndrome;

use self::frame::TrapFrame;
use self::syndrome::Syndrome;
use crate::cpu;
use crate::helper::{FAR_EL2, HPFAR_EL2};
use crate::realm::context::Context;
use crate::rmi;
use monitor::realm::vcpu::VCPU;

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
            undefined => {
                panic!("{:?} and {:?} on CPU {:?}", info, esr, cpu::id());
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

/// This function is called when an exception occurs from LowerAArch64.
/// To enter RMM (EL2), return 1. Otherwise, return 0 to go back to EL1.
/// The `info` parameter specifies source (first 16 bits) and kind (following 16
/// bits) of the exception.
/// The `esr` has the value of a syndrome register (ESR_ELx) holding the cause
/// of the Synchronous and SError exception.
/// The `vcpu` has the VCPU context.
/// The `tf` has the TrapFrame of current context.
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn handle_lower_exception(
    info: Info,
    esr: u32,
    vcpu: &mut VCPU<Context>,
    tf: &mut TrapFrame,
) -> u64 {
    match info.kind {
        Kind::Synchronous => match Syndrome::from(esr) {
            Syndrome::HVC => 1,
            Syndrome::InstructionAbort(_) | Syndrome::DataAbort(_) => {
                tf.regs[0] = rmi::RET_PAGE_FAULT as u64;
                tf.regs[1] = unsafe {
                    HPFAR_EL2.get_masked_value(HPFAR_EL2::FIPA) << 12
                        | FAR_EL2.get_masked(FAR_EL2::OFFSET)
                };
                1
            }
            undefined => {
                debug!("{:?} and {:X?} on CPU {:?}", info, esr, cpu::id());
                debug!("{:#X?}", vcpu);
                0
            }
        },
        _ => {
            warn!(
                "Unknown exception! Info={:?}, ESR={:x} on CPU {:?}",
                info,
                esr,
                cpu::id()
            );
            0
        }
    }
}
