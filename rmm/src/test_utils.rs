use crate::event::Context;
use crate::event::Mainloop;
use crate::granule::array::granule_addr; // alloc_granule
use crate::monitor::Monitor;
use crate::rmi::realm::Params;
use crate::rmi::{GRANULE_DELEGATE, REALM_CREATE, SUCCESS};

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

use crate::{get_granule, get_granule_if};

pub fn realm_create() -> usize {
    for mocking_addr in &[granule_addr(0), granule_addr(1)] {
        let ret = rmi::<GRANULE_DELEGATE>(&[*mocking_addr]);
        assert_eq!(ret[0], SUCCESS);
    }

    let (rd, rtt, params_ptr) = (granule_addr(0), granule_addr(1), granule_addr(2));

    unsafe {
        let params = &mut *(params_ptr as *mut Params);
        params.s2sz = 40;
        params.rtt_num_start = 1;
        params.rtt_level_start = 0;
        params.rtt_base = rtt as u64;
    };

    let ret = rmi::<REALM_CREATE>(&[rd, params_ptr]);
    assert_eq!(ret[0], SUCCESS);

    rd
}
