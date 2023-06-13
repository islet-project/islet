use core::arch::asm;
use monitor::smc::Code;

// defined in trusted-firmware-a/include/services/rmmd_svc.h
pub const SMC_ASC_MARK_REALM: usize = 0xc400_01b0;
pub const SMC_ASC_MARK_NONSECURE: usize = 0xc400_01b1;

#[derive(Debug)]
pub struct SMC;

impl SMC {
    pub fn new() -> &'static SMC {
        &SMC {}
    }
}

impl monitor::smc::Caller for SMC {
    fn convert(&self, command: Code) -> usize {
        match command {
            Code::MarkRealm => SMC_ASC_MARK_REALM,
            Code::MarkNonSecure => SMC_ASC_MARK_NONSECURE,
        }
    }

    fn call(&self, command: usize, args: &[usize]) -> [usize; 8] {
        let mut ret: [usize; 8] = [0usize; 8];
        let mut padded_args: [usize; 8] = [0usize; 8];
        let start = 1;
        let end = start + args.len();

        if end > ret.len() - 1 {
            // TODO: need a more graceful way to return error value (Result?)
            error!("{} arguments exceed the current limit of smc call. Please try assigning more registers to smc", args.len());
            return ret;
        }

        let put = |arr: &mut [usize; 8]| {
            arr[0] = command;
            arr[start..end].copy_from_slice(args);
        };
        put(&mut ret);
        put(&mut padded_args);

        // TODO: support more number of registers than 8 if needed
        unsafe {
            asm!(
                "smc #0x0",
                inlateout("x0") padded_args[0] => ret[0],
                inlateout("x1") padded_args[1] => ret[1],
                inlateout("x2") padded_args[2] => ret[2],
                inlateout("x3") padded_args[3] => ret[3],
                inlateout("x4") padded_args[4] => ret[4],
                inlateout("x5") padded_args[5] => ret[5],
                inlateout("x6") padded_args[6] => ret[6],
                inlateout("x7") padded_args[7] => ret[7],
            )
        }

        trace!("cmd[{:X}], args{:X?}, ret{:X?}", command, args, ret);
        ret
    }
}
