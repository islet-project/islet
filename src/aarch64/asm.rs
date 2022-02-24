#[inline(always)]
pub fn brk(b: u16) {
    unsafe {
        llvm_asm! {
            "brk $0"
            : : "i"(b) : : "volatile"
        }
    }
}

#[inline(always)]
pub fn eret() {
    unsafe {
        llvm_asm! {
            "eret"
            : : : : "volatile"
        }
    }
}
