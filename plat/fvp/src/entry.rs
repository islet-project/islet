use crate::allocator;
use crate::log::LevelFilter;

use armv9a::regs::*;
use core::ptr::{addr_of, addr_of_mut};
use islet_rmm::config::{NUM_OF_CPU, RMM_STACK_SIZE};
use islet_rmm::io::stdout;
use islet_rmm::logger;

#[no_mangle]
#[link_section = ".stack"]
static mut RMM_STACK: [u8; RMM_STACK_SIZE * NUM_OF_CPU] = [0; RMM_STACK_SIZE * NUM_OF_CPU];

#[naked]
#[link_section = ".head.text"]
#[no_mangle]
unsafe extern "C" fn rmm_entry() -> ! {
    core::arch::asm!("
        msr spsel, #1
        bl get_cpu_id

        ldr x1, =__RMM_STACK_END__
        mov x2, {}
        mul x0, x0, x2
        sub x0, x1, x0
        mov sp, x0

        bl setup

        1:
        bl main
        b 1b",
        const RMM_STACK_SIZE,
        options(noreturn)
    )
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
    const UART3_BASE: usize = 0x1c0c_0000usize;
    let _ = stdout().attach(uart::pl011::device(UART3_BASE));
    logger::register_global_logger(LevelFilter::Trace); // Control log level
    info!("Initialized the console!");
}

/// Initialize the memory management configuration.
/// This function is called once in cold boot.
unsafe fn init_mm() {
    // Assert 4KB granules are supported.
    assert_eq!(
        ID_AA64MMFR0_EL1.get_masked_value(ID_AA64MMFR0_EL1::TGran4),
        0,
        "4KB granules are not supported"
    );

    // Assert ID_AA64MMFR0_EL1::PARange
    let pa_bits_table = [32, 36, 40, 42, 44, 48, 52];
    let pa = ID_AA64MMFR0_EL1.get_masked_value(ID_AA64MMFR0_EL1::PARange) as usize;
    let pa_range = pa_bits_table[pa]; // Panic if pa > 6
    info!("pa range is {}", pa_range);
}

#[no_mangle]
#[allow(unused)]
unsafe fn setup() {
    static mut COLD_BOOT: bool = true;

    if (addr_of!(COLD_BOOT) as *const bool).read_volatile() {
        clear_bss();
        allocator::init();
        init_console();
        init_mm();

        (addr_of_mut!(COLD_BOOT) as *mut bool).write_volatile(false);
    }
}
