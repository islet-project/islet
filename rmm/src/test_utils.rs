use crate::event::Context;
use crate::event::Mainloop;
use crate::monitor::Monitor;
use alloc::vec::Vec;

pub fn rmi<const COMMAND: usize>(arg: &[usize]) -> Vec<usize> {
    let mut mainloop = Mainloop::new();
    let monitor = Monitor::new();
    (&mut mainloop).add_event_handlers();

    let mut ctx = Context::new(COMMAND);
    ctx.init_arg(&arg);
    ctx.init_ret(&[0; 8]);

    let handler = mainloop.on_event.get(&COMMAND).unwrap();
    if let Err(code) = handler(&ctx.arg, &mut ctx.ret, &monitor) {
        ctx.ret[0] = code.into();
    }
    ctx.ret.to_vec()
}

pub fn extract_bits(value: usize, start: u32, end: u32) -> usize {
    let num_bits = end - start + 1;
    let mask = if num_bits == usize::BITS {
        usize::MAX
    } else {
        (1 << num_bits) - 1
    };
    (value >> start) & mask
}
