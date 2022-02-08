use realm_management_monitor::io::{stdout, Write as IoWrite};
use realm_management_monitor::println;

use crate::allocator;
use crate::config::RMM_STACK_SIZE;

extern crate alloc;

#[no_mangle]
#[link_section = ".stack"]
static mut RMM_STACK: [u8; RMM_STACK_SIZE] = [0; RMM_STACK_SIZE];

#[naked]
#[link_section = ".head.text"]
#[no_mangle]
unsafe extern "C" fn rmm_entry() {
    #![allow(unsupported_naked_functions)]
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

fn init_console() {
    let _ = stdout().attach(crate::driver::uart::pl011::device());

    println!("RMM: initialized the console!");
}

#[no_mangle]
#[allow(unused)]
unsafe fn setup() {
    static mut COLD_BOOT: bool = true;

    if (&COLD_BOOT as *const bool).read_volatile() {
        clear_bss();
        allocator::init();

        init_console();

        (&mut COLD_BOOT as *mut bool).write_volatile(false);
    }
}
