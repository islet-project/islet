use realm_management_monitor::io::{stdout, Write};

use crate::alloc::init_heap;
use crate::config::RMM_STACK_SIZE;

#[no_mangle]
#[link_section = ".stack"]
static mut RMM_STACK: [u8; RMM_STACK_SIZE] = [0; RMM_STACK_SIZE];

#[link_section = ".head.text"]
#[no_mangle]
unsafe extern "C" fn rmm_entry() -> ! {
    llvm_asm! {
        "
		ldr x0, =__RMM_STACK_END__
		mov sp, x0

		bl setup

		1:
		bl main
		b 1b
        "
        : : : : "volatile"
    }

    loop {}
}

extern "C" {
    static __BSS_START__: usize;
    static __BSS_SIZE__: usize;
}

unsafe fn clear_bss() {
    let bss = core::slice::from_raw_parts_mut(
        &__BSS_START__ as *const usize as *mut u64,
        &__BSS_SIZE__ as *const usize as usize / core::mem::size_of::<u64>(),
    );
    bss.fill(0);
}

unsafe fn init_console() {
    let _ = stdout().attach(crate::driver::uart::pl011::device());

    let _ = stdout().write_all("RMM: initialized the console!\n".as_bytes());
}

#[no_mangle]
#[allow(unused)]
unsafe fn setup() {
    static mut COLD_BOOT: bool = true;

    if (&COLD_BOOT as *const bool).read_volatile() {
        clear_bss();
        init_heap();

        init_console();

        (&mut COLD_BOOT as *mut bool).write_volatile(false);
    }
}
