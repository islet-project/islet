use super::gpt::{mark_ns, mark_realm};

use crate::event::Mainloop;
use crate::listen;
use crate::rmi;

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::RTT_INIT_RIPAS, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::RTT_READ_ENTRY, |ctx, _, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::DATA_CREATE, |ctx, rmi, smc| {
        // taget_pa: location where realm data is created.
        let taget_pa = ctx.arg[0];
        let rd = ctx.arg[1];
        let ipa = ctx.arg[2];
        let src_pa = ctx.arg[3];

        // islet stores rd as realm id
        let realm_id = rd;
        let granule_sz = 4096;

        // TODO: Make sure DATA_CREATE is only processed
        // when the realm is in its New state.

        // 1. map src to rmm
        // FIXME: replace delegation with mapping a page to rmm
        if mark_realm(smc, src_pa)[0] != 0 {
            ctx.ret[0] = rmi::RET_FAIL;
            return;
        }

        // TODO: 2. map src to rmm

        // 3. copy src to _data
        unsafe {
            core::ptr::copy_nonoverlapping(src_pa as *const u8, taget_pa as *mut u8, granule_sz);
        }
        // TODO: Make sure DATA_CREATE is only processed
        // when the realm is in its New state.

        // 4. map ipa to _taget_pa into S2 table
        let prot = rmi::MapProt::new(0);
        let ret = rmi.map(realm_id, ipa, taget_pa, granule_sz, prot.get());
        match ret {
            Ok(_) => ctx.ret[0] = rmi::SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }

        // TODO: 5. perform measure

        // 6. unmap src and _taget_pa from rmm
        // FIXME: replace undelegation with unmapping from rmm
        if mark_ns(smc, src_pa)[0] != 0 {
            ctx.ret[0] = rmi::RET_FAIL;
        }
    });

    // Map an unprotected IPA to a non-secure PA.
    listen!(mainloop, rmi::RTT_MAP_UNPROTECTED, |ctx, rmi, _| {
        let rd = ctx.arg[0];
        let ipa = ctx.arg[1];
        let _level = ctx.arg[2];
        let ns_pa = ctx.arg[3];

        // islet stores rd as realm id
        let realm_id = rd;
        let granule_sz = 4096;
        let mut prot = rmi::MapProt(0);
        prot.set_bit(rmi::MapProt::NS_PAS);
        let _ret = rmi.map(realm_id, ipa, ns_pa, granule_sz, prot.get());
        ctx.ret[0] = rmi::SUCCESS;
    });
}
