const STACK_SIZE: usize = 0x1000;

#[no_mangle]
#[link_section = ".stack"]
static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
