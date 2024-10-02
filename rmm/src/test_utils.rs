use crate::event::Context;
use crate::event::Mainloop;
use crate::granule::{GRANULE_REGION, GRANULE_SIZE};
use crate::monitor::Monitor;
use crate::rmi::realm::Params as RealmParams;
use crate::rmi::rec::params::Params as RecParams;
use crate::rmi::{GRANULE_DELEGATE, GRANULE_UNDELEGATE, REALM_CREATE, REALM_DESTROY, SUCCESS};
use crate::rmi::{MAX_REC_AUX_GRANULES, REC_AUX_COUNT, REC_CREATE, REC_DESTROY};
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
        let params = &mut *(params_ptr as *mut RealmParams);
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

pub fn rec_create(rd: usize) -> usize {
    let (rec, params_ptr) = (alloc_granule(IDX_REC), alloc_granule(IDX_REC_PARAMS));
    let ret = rmi::<GRANULE_DELEGATE>(&[rec]);
    assert_eq!(ret[0], SUCCESS);

    let ret = rmi::<REC_AUX_COUNT>(&[rd]);
    assert_eq!(ret[0], SUCCESS);
    assert_eq!(ret[1], MAX_REC_AUX_GRANULES);

    let aux_count = ret[1];
    unsafe {
        let params = &mut *(params_ptr as *mut RecParams);
        params.pc = 0;
        params.flags = 1; // RMI_RUNNABLE
        params.mpidr = 0; // MPIDR_VALID
        params.num_aux = aux_count as u64;

        for idx in 0..aux_count {
            let mocking_addr = alloc_granule(idx + IDX_REC_AUX);
            let ret = rmi::<GRANULE_DELEGATE>(&[mocking_addr]);
            assert_eq!(ret[0], SUCCESS);

            params.aux[idx] = mocking_addr as u64;
        }
    }

    let ret = rmi::<REC_CREATE>(&[rd, rec, params_ptr]);
    assert_eq!(ret[0], SUCCESS);

    rec
}

pub fn rec_destroy(rec: usize) {
    let ret = rmi::<REC_DESTROY>(&[rec]);
    assert_eq!(ret[0], SUCCESS);

    let ret = rmi::<GRANULE_UNDELEGATE>(&[rec]);
    assert_eq!(ret[0], SUCCESS);

    for idx in 0..MAX_REC_AUX_GRANULES {
        let mocking_addr = alloc_granule(idx + IDX_REC_AUX);
        let ret = rmi::<GRANULE_UNDELEGATE>(&[mocking_addr]);
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

/*
 * 0: RD
 * 1: RTT0
 * 2: Realm Params
 * 3: RTT1
 * 4: RTT2
 * 5: RTT3
 * 6: RTT4
 * 7: REC
 * 8: REC Params
 * 9..25: REC AUX
 */
const IDX_REC: usize = 7;
const IDX_REC_PARAMS: usize = 8;
const IDX_REC_AUX: usize = 9;
pub fn alloc_granule(idx: usize) -> usize {
    let start = unsafe { GRANULE_REGION.as_ptr() as usize };
    let first = crate::test_utils::align_up(start);
    first + idx * GRANULE_SIZE
}
