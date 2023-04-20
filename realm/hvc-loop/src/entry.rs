const STACK_SIZE: usize = 0x1000;

#[no_mangle]
#[link_section = ".stack"]
static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

#[link_section = ".head.text"]
#[no_mangle]
unsafe extern "C" fn _entry() -> ! {
    core::arch::asm!(
        "
        ldr x0, =__STACK_END__
        mov sp, x0

        bl setup

        1:
        bl main
        b 1b",
        options(noreturn)
    )
}

#[no_mangle]
unsafe fn setup() {
    extern "C" {
        static mut __BSS_START__: usize;
        static mut __BSS_SIZE__: usize;
    }

    clear_bss(&mut __BSS_START__, &mut __BSS_SIZE__);
}

unsafe fn clear_bss(mut sbss: *mut usize, ebss: *mut usize) {
    while sbss < ebss {
        core::ptr::write_volatile(sbss, core::mem::zeroed());
        sbss = sbss.offset(1);
    }
}
