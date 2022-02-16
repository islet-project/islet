pub const SMC_ASC_MARK_REALM: usize = 0xc400_0100;
pub const SMC_ASC_MARK_NONSECURE: usize = 0xc400_0101;

pub fn call(args: [usize; 5]) -> [usize; 5] {
    let mut ret: [usize; 5] = [0usize; 5];

    unsafe {
        llvm_asm! {
            "smc #0x0"
            : "={x0}"(ret[0]), "={x1}"(ret[1]), "={x2}"(ret[2]),
                 "={x3}"(ret[3]), "={x4}"(ret[4])
            : "{x0}"(args[0]), "{x1}"(args[1]), "{x2}"(args[2]),
                 "{x3}"(args[3]),"{x4}"(args[4]) : : "volatile"
        }
    }
    ret
}
