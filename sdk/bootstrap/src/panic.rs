#[panic_handler]
pub extern "C" fn panic_handler(_info: &core::panic::PanicInfo<'_>) -> ! {
    halt()
}

pub fn halt() -> ! {
    loop {}
}
