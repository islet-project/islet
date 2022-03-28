use monitor::eprintln;
use monitor::io::Write as IoWrite;

#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    panic!("OOM! memory allocation of {} bytes failed", _layout.size())
}

#[panic_handler]
pub extern "C" fn panic_handler(_info: &core::panic::PanicInfo<'_>) -> ! {
    eprintln!("RMM: {}", _info);
    halt()
}

pub fn halt() -> ! {
    loop {}
}
