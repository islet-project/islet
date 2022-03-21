use rmm_core::{eprintln, io::Write as IoWrite};

mod frame;
mod syndrome;

use self::frame::TrapFrame;
use self::syndrome::Syndrome;
use crate::realm::context::Context;
use rmm_core::realm::vcpu::VCPU;

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
                eprintln!("brk #{}", b);
                eprintln!("{:?}\nESR: {:X}\n{:#X?}", info, esr, tf);
                tf.elr += 4; //continue
            }
            undefined => {
                panic!("{:?} and {:?}", info, undefined);
            }
        },
        _ => {
            panic!("Unknown exception! Info={:?}, ESR={:x}", info, esr);
        }
    }
}

/// This function is called when an exception occurs from LowerAArch64.
/// The `info` parameter specifies source (first 16 bits) and kind (following 16
/// bits) of the exception.
/// The `esr` has the value of a syndrome register (ESR_ELx) holding the cause
/// of the Synchronous and SError exception.
/// The `vcpu` has the VCPU context.
#[no_mangle]
#[allow(unused_variables)]
pub extern "C" fn handle_lower_exception(info: Info, esr: u32, vcpu: &mut VCPU<Context>) -> u64 {
    // eprintln!("{:?}\nESR: {:X}\n{:#X?}", info, esr, vcpu);
    // TODO: Handle exceptions properly
    let cmd = usize::from(crate::rmi::Code::RequestComplete);
    let arg = [0; 4];
    crate::smc::call(cmd, arg);
    0
}
