use core::arch::asm;

#[inline(always)]
pub fn brk<const IMM: u16>() {
    unsafe {
        asm!("brk #{}", const IMM);
    }
}

#[inline(always)]
pub fn eret() {
    unsafe {
        asm!("eret");
    }
}

#[inline(always)]
pub fn smc<const IMM: u16>() {
    unsafe {
        asm!("smc #{}", const IMM);
    }
}

#[inline(always)]
pub fn hvc<const IMM: u16>() {
    unsafe {
        asm!("hvc #{}", const IMM);
    }
}
