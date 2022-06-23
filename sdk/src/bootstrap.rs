//To prevent LTO, this has to be mutable
// no_mangle is a suggested workaround for https://github.com/rust-lang/rust/issues/31758
#[allow(dead_code)]
#[no_mangle]
#[link_section = ".bootstrap"]
static mut BINARY: [u8; 4096] = [0u8; 4096];

pub fn get_binary() -> &'static [u8] {
    unsafe { &BINARY }
}
