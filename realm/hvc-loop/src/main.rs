#![no_std]
#![no_main]
#![feature(asm_const)]
#![warn(rust_2018_idioms)]

mod entry;
mod panic;

use core::arch::asm;

#[no_mangle]
pub unsafe fn main() -> ! {
    // CHECK:
    //   HOST_CALL is not initialized when use tf-rmm
    //   HOST_CALL is initialized when use islet-rmm
    HOST_CALL.padding = 0;

    HOST_CALL.imm = 1;
    let arg = [
        &HOST_CALL as *const _ as usize,
        HOST_CALL.imm as usize,
        0,
        0,
    ];
    let ret = smc(RSI_HOST_CALL, arg);

    HOST_CALL.imm = 2;
    let arg = [
        &HOST_CALL as *const _ as usize,
        HOST_CALL.imm as usize,
        0,
        0,
    ];
    let ret = smc(RSI_HOST_CALL, arg);

    loop {}
}

// TODO:
//   Detach rmm-spec(data structures & commands) to newly crate.
//   And use it both rmm and realm
const RSI_HOST_CALL: usize = 0xC400_0199;

#[repr(C)]
pub struct HostCall {
    pub imm: u16,
    pub padding: u16,
}

static mut HOST_CALL: HostCall = HostCall { imm: 0, padding: 0 };

unsafe fn smc(cmd: usize, arg: [usize; 4]) -> [usize; 8] {
    let mut ret: [usize; 8] = [0usize; 8];
    asm! {
        "smc #0x0",
        inlateout("x0") cmd => ret[0],
        inlateout("x1") arg[0] => ret[1],
        inlateout("x2") arg[1] => ret[2],
        inlateout("x3") arg[2] => ret[3],
        inlateout("x4") arg[3] => ret[4],
        out("x5") ret[5],
        out("x6") ret[6],
        out("x7") ret[7],
    }
    ret
}
