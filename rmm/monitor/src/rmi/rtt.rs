use super::realm::{rd::State, Rd};

use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::rmm::granule;
use crate::rmm::granule::{GranuleState, RmmGranule};

extern crate alloc;

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::RTT_INIT_RIPAS, |ctx, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::RTT_READ_ENTRY, |ctx, _| {
        super::dummy();
        ctx.ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::DATA_CREATE, |ctx, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;
        // taget_pa: location where realm data is created.
        let taget_pa = ctx.arg[0];
        let rd = unsafe { Rd::into(ctx.arg[1]) };
        let ipa = ctx.arg[2];
        let src_pa = ctx.arg[3];

        let realm_id = rd.id();
        let granule_sz = 4096;

        // Make sure DATA_CREATE is only processed
        // when the realm is in its New state.
        if !rd.at_state(State::New) {
            ctx.ret[0] = rmi::RET_FAIL;
            return;
        }

        // 1. map src to rmm
        mm.map(src_pa, false);
        let g_ipa = granule::find_granule(ipa, GranuleState::Delegated);
        let g_tar = granule::find_granule(taget_pa, GranuleState::Delegated);
        g_ipa.set_state(GranuleState::Data, mm);
        g_tar.set_state(GranuleState::Data, mm);

        // 3. copy src to _data
        unsafe {
            core::ptr::copy_nonoverlapping(src_pa as *const u8, taget_pa as *mut u8, granule_sz);
        }

        // 4. map ipa to _taget_pa into S2 table
        let prot = rmi::MapProt::new(0);
        let ret = rmi.map(realm_id, ipa, taget_pa, granule_sz, prot.get());
        match ret {
            Ok(_) => ctx.ret[0] = rmi::SUCCESS,
            Err(_) => ctx.ret[0] = rmi::RET_FAIL,
        }

        // TODO: 5. perform measure

        // 6. unmap src and _taget_pa from rmm
        mm.unmap(src_pa);
    });

    listen!(mainloop, rmi::DATA_DESTORY, |ctx, rmm| {
        let mm = rmm.mm;
        let target_data = ctx.arg[0];
        let ipa = ctx.arg[1];

        let g_tar = granule::find_granule(target_data, GranuleState::Data);
        let g_ipa = granule::find_granule(ipa, GranuleState::Data);
        g_tar.set_state(GranuleState::Delegated, mm);
        g_ipa.set_state(GranuleState::Delegated, mm);

        ctx.ret[0] = rmi::SUCCESS;
    });

    // Map an unprotected IPA to a non-secure PA.
    listen!(mainloop, rmi::RTT_MAP_UNPROTECTED, |ctx, rmm| {
        let rmi = rmm.rmi;
        let rd = unsafe { Rd::into(ctx.arg[0]) };
        let ipa = ctx.arg[1];
        let _level = ctx.arg[2];
        let ns_pa = ctx.arg[3];

        // islet stores rd as realm id
        let realm_id = rd.id();
        let granule_sz = 4096;
        let mut prot = rmi::MapProt(0);
        prot.set_bit(rmi::MapProt::NS_PAS);
        let _ret = rmi.map(realm_id, ipa, ns_pa, granule_sz, prot.get());
        ctx.ret[0] = rmi::SUCCESS;
    });
}
