#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    halt();
}

#[panic_handler]
pub extern "C" fn panic_handler(_info: &core::panic::PanicInfo<'_>) -> ! {
    halt();
}

#[no_mangle]
pub fn halt() -> ! {
    loop {}
}
