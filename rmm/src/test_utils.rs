use crate::event::Context;
use crate::event::Mainloop;
use crate::granule::{GRANULE_REGION, GRANULE_SIZE};
use crate::monitor::Monitor;
use crate::rmi::realm::Params;
use crate::rmi::{GRANULE_DELEGATE, GRANULE_UNDELEGATE, REALM_CREATE, REALM_DESTROY, SUCCESS};
use crate::{get_granule, get_granule_if};

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

pub fn realm_create() -> usize {
    for mocking_addr in &[alloc_granule(0), alloc_granule(1)] {
        let ret = rmi::<GRANULE_DELEGATE>(&[*mocking_addr]);
        assert_eq!(ret[0], SUCCESS);
    }

    let (rd, rtt, params_ptr) = (alloc_granule(0), alloc_granule(1), alloc_granule(2));

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

pub fn realm_destroy(rd: usize) {
    let ret = rmi::<REALM_DESTROY>(&[rd]);
    assert_eq!(ret[0], SUCCESS);

    for mocking_addr in &[alloc_granule(0), alloc_granule(1)] {
        let ret = rmi::<GRANULE_UNDELEGATE>(&[*mocking_addr]);
        assert_eq!(ret[0], SUCCESS);
    }
}

pub fn align_up(addr: usize) -> usize {
    let align_mask = GRANULE_SIZE - 1;
    if addr & align_mask == 0 {
        addr
    } else {
        (addr | align_mask) + 1
    }
}

pub fn alloc_granule(idx: usize) -> usize {
    let start = unsafe { GRANULE_REGION.as_ptr() as usize };
    let first = crate::test_utils::align_up(start);
    first + idx * GRANULE_SIZE
}
