#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    panic!("OOM! memory allocation of {} bytes failed", _layout.size())
}

#[panic_handler]
pub extern "C" fn panic_handler(_info: &core::panic::PanicInfo<'_>) -> ! {
    error!("RMM: {}", _info);
    halt()
}

pub fn halt() -> ! {
    loop {}
}
