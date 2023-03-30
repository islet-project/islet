mod frame;
pub mod syndrome;

use self::frame::TrapFrame;
use self::syndrome::{Fault, Syndrome};
use crate::cpu;
use crate::helper::{ESR_EL2, FAR_EL2, HPFAR_EL2};
use crate::realm::context::Context;
use crate::rsi::RSI_REMAP_PAGE;

use monitor::realm::vcpu::VCPU;
use monitor::{rmi, rsi};

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
        // TODO: adjust elr according to the decision that kvm made
        Kind::Synchronous => match Syndrome::from(esr) {
            Syndrome::HVC => {
                debug!("Synchronous: HVC");
                tf.regs[0] = rmi::RET_EXCEPTION_TRAP as u64;
                tf.regs[1] = esr as u64;
                tf.regs[2] = 0;
                tf.regs[3] = unsafe { FAR_EL2.get() };
                1 // forward to nw w/o increasing elr
            }
            Syndrome::SMC => {
                debug!("Synchronous: SMC: {:#X}", vcpu.context.gp_regs[0]);
                let req_id: u64 = vcpu.context.gp_regs[0];
                match req_id as usize {
                    rsi::HOST_CALL => {
                        tf.regs[0] = rsi::HOST_CALL as u64;
                        tf.regs[1] = vcpu.context.gp_regs[1];
                        tf.regs[2] = vcpu.context.gp_regs[2];
                        tf.regs[3] = vcpu.context.gp_regs[3];
                        1
                    }
                    RSI_REMAP_PAGE => {
                        // fabricate an exception to force remap page as shared from non-sec
                        tf.regs[0] = rmi::RET_EXCEPTION_TRAP as u64;
                        tf.regs[1] = Syndrome::DataAbort(Fault::Translation { level: 3 }).into();
                        tf.regs[2] = vcpu.context.gp_regs[1] >> 8;
                        tf.regs[3] = vcpu.context.gp_regs[2];
                        vcpu.context.elr += 4; // continue
                        unsafe {
                            ESR_EL2.set(tf.regs[1]);
                            HPFAR_EL2.set(tf.regs[2]);
                            FAR_EL2.set(tf.regs[3]);
                        }
                        1 // forward to nw w/o increasing elr
                    }
                    _ => {
                        vcpu.context.elr += 4; // continue
                        0 // for now, trap to el2 to check the realm progress
                    }
                }
                //vcpu.context.elr += 4; // continue
                //0 // for now, trap to el2 to check the realm progress
            }
            Syndrome::InstructionAbort(_) | Syndrome::DataAbort(_) => {
                debug!("Synchronous: InstructionAbort | DataAbort");
                tf.regs[0] = rmi::RET_EXCEPTION_TRAP as u64;
                tf.regs[1] = esr as u64;
                tf.regs[2] = unsafe { HPFAR_EL2.get() };
                tf.regs[3] = unsafe { FAR_EL2.get() };
                1
            }
            undefined => {
                debug!("Synchronous: Other");
                tf.regs[0] = rmi::RET_EXCEPTION_TRAP as u64;
                tf.regs[1] = esr as u64;
                tf.regs[2] = unsafe { HPFAR_EL2.get() };
                tf.regs[3] = unsafe { FAR_EL2.get() };
                1
            }
        },
        Kind::Irq => {
            debug!("IRQ");
            tf.regs[0] = rmi::RET_EXCEPTION_IRQ as u64;
            tf.regs[1] = esr as u64;
            tf.regs[2] = 0;
            tf.regs[3] = unsafe { FAR_EL2.get() };
            1
        }
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
