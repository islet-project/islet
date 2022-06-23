// defined in trusted-firmware-a/include/services/rmmd_svc.h
const SMC_ASC_MARK_REALM: usize = 0xc400_01b0;
const SMC_ASC_MARK_NONSECURE: usize = 0xc400_01b2;

pub enum Code {
    MarkRealm,
    MarkNonSecure,
}

impl From<Code> for usize {
    fn from(origin: Code) -> Self {
        match origin {
            Code::MarkRealm => SMC_ASC_MARK_REALM,
            Code::MarkNonSecure => SMC_ASC_MARK_NONSECURE,
        }
    }
}

pub fn call(command: usize, args: [usize; 4]) -> [usize; 8] {
    let mut ret: [usize; 8] = [0usize; 8];

    unsafe {
        llvm_asm! {
            "smc #0x0"
            : "={x0}"(ret[0]), "={x1}"(ret[1]), "={x2}"(ret[2]),
                 "={x3}"(ret[3]), "={x4}"(ret[4]),
                 "={x5}"(ret[5]), "={x6}"(ret[6]), "={x7}"(ret[7])
            : "{x0}"(command), "{x1}"(args[0]), "{x2}"(args[1]),
                 "{x3}"(args[2]),"{x4}"(args[3]) : : "volatile"
        }
    }
    ret
}
