pub const RMM_REQ_COMPLETE: usize = 0xc0000010;

#[inline(always)]
unsafe fn smc(x0: usize, x1: usize) {
    llvm_asm! {
        "
		smc #0x0
		"
        : : "{x0}"(x0),"{x1}"(x1) : : "volatile"
    }
}

pub fn rmm_exit() {
    unsafe {
        smc(RMM_REQ_COMPLETE, 0);
    }
}
