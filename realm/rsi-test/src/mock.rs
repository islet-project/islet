use core::arch::asm;

// TODO:
//   Detach rmm-spec(data structures & commands) to newly crate.
//   And use it both rmm and realm
const RSI_HOST_CALL: usize = 0xC400_0199;
const CMD_GET_SHARED_BUF: u16 = 1;
const CMD_SUCCESS: u16 = 2;

#[repr(C)]
struct HostCall {
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

pub unsafe fn get_ns_buffer() {
    // CHECK:
    //   HOST_CALL is not initialized when use tf-rmm
    //   HOST_CALL is initialized when use islet-rmm
    HOST_CALL.padding = 0;
    HOST_CALL.imm = CMD_GET_SHARED_BUF;
    let arg = [
        &HOST_CALL as *const _ as usize,
        HOST_CALL.imm as usize,
        0,
        0,
    ];
    let _ = smc(RSI_HOST_CALL, arg);
}

pub unsafe fn exit_to_host() {
    HOST_CALL.imm = CMD_SUCCESS;
    let arg = [
        &HOST_CALL as *const _ as usize,
        HOST_CALL.imm as usize,
        0,
        0,
    ];
    let _ = smc(RSI_HOST_CALL, arg);
}
