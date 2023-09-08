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

#[inline(always)]
pub fn dcache_flush(addr: usize, len: usize) {
    let mut cur_addr = addr;
    let addr_end = addr + len;
    unsafe {
        while cur_addr < addr_end {
            asm!("dc civac, {}", in(reg) cur_addr);
            asm!("dsb ish");
            asm!("isb");
            cur_addr += 64; // the cache line size is 64
        }
    }
}
