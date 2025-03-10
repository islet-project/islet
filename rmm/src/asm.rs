use core::arch::asm;

pub const SMC_SUCCESS: usize = 0;
#[cfg(any(kani, miri, test, fuzzing))]
pub const SMC_ERROR: usize = 1;

pub fn smc(cmd: usize, args: &[usize]) -> [usize; 8] {
    let mut ret: [usize; 8] = [0usize; 8];
    let mut padded_args: [usize; 8] = [0usize; 8];
    let start = 1;
    let end = start + args.len();

    if end > ret.len() - 1 {
        // TODO: need a more graceful way to return error value (Result?)
        error!("{} arguments exceed the current limit of smc call. Please try assigning more registers to smc", args.len());
        return ret;
    }

    let put = |arr: &mut [usize; 8]| {
        arr[0] = cmd;
        arr[start..end].copy_from_slice(args);
    };
    put(&mut ret);
    put(&mut padded_args);
    #[cfg(any(kani, miri, test, fuzzing))]
    if cmd == crate::rmi::gpt::MARK_REALM {
        use crate::get_granule;
        use crate::granule::entry::GranuleGpt;
        let addr = args[0];
        let gpt = get_granule!(addr).map(|guard| guard.gpt).unwrap();
        if gpt != GranuleGpt::GPT_NS {
            ret[0] = SMC_ERROR;
        } else {
            let _ = get_granule!(addr).map(|mut guard| guard.set_gpt(GranuleGpt::GPT_REALM));
            ret[0] = SMC_SUCCESS;
        }
    } else if cmd == crate::rmi::gpt::MARK_NONSECURE {
        use crate::get_granule;
        use crate::granule::entry::GranuleGpt;
        let addr = args[0];
        let is_valid = get_granule!(addr).map(|guard| guard.is_valid()).unwrap();
        assert!(is_valid);
        let gpt = get_granule!(addr).map(|guard| guard.gpt).unwrap();
        if gpt != GranuleGpt::GPT_REALM {
            ret[0] = SMC_ERROR;
        } else {
            let _ = get_granule!(addr).map(|mut guard| guard.set_gpt(GranuleGpt::GPT_NS));
            ret[0] = SMC_SUCCESS;
        }
    }

    // TODO: support more number of registers than 8 if needed
    #[cfg(not(any(kani, miri, test, fuzzing)))]
    unsafe {
        asm!(
            "smc #0x0",
            inlateout("x0") padded_args[0] => ret[0],
            inlateout("x1") padded_args[1] => ret[1],
            inlateout("x2") padded_args[2] => ret[2],
            inlateout("x3") padded_args[3] => ret[3],
            inlateout("x4") padded_args[4] => ret[4],
            inlateout("x5") padded_args[5] => ret[5],
            inlateout("x6") padded_args[6] => ret[6],
            inlateout("x7") padded_args[7] => ret[7],
        )
    }

    ret
}

#[inline(always)]
pub fn dcache_flush(addr: usize, len: usize) {
    let mut cur_addr = addr;
    let addr_end = addr + len;
    unsafe {
        while cur_addr < addr_end {
            asm!("dc civac, {}", in(reg) cur_addr);
            asm!("dsb ish");
            asm!("isb");
            cur_addr += 64; // the cache line size is 64
        }
    }
}
