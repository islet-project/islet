use super::realm::{rd::State, Rd};
use super::rec::Rec;

use crate::event::Mainloop;
use crate::listen;
use crate::rmi;
use crate::rmm::granule;
use crate::rmm::granule::GranuleState;

extern crate alloc;

const RIPAS_EMPTY: u64 = 0;
const RIPAS_RAM: u64 = 1;

fn level_to_size(level: usize) -> u64 {
    match level {
        0 => 512 << 30,
        1 => 1 << 30,
        2 => 2 << 20,
        3 => 1 << 12,
        _ => 0,
    }
}

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::RTT_INIT_RIPAS, |_, ret, _| {
        super::dummy();
        ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::RTT_SET_RIPAS, |arg, ret, rmm| {
        let _rmi = rmm.rmi;
        let _rd = unsafe { Rd::into(arg[0]) };
        let mut rec = unsafe { Rec::into(arg[1]) };
        let _ipa = arg[2];
        let level = arg[3];
        let ripas = arg[4];

        let mut prot = rmi::MapProt::new(0);
        match ripas as u64 {
            RIPAS_EMPTY => prot.set_bit(rmi::MapProt::NS_PAS),
            RIPAS_RAM => { /* do nothing: ripas ram by default */ }
            _ => {
                warn!("Unknown RIPAS:{}", ripas);
                ret[0] = rmi::RET_FAIL;
                return;
            }
        }
        // TODO: update mapping
        super::dummy();
        let map_size = level_to_size(level);
        rec.inc_ripas_addr(map_size);
        ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::RTT_READ_ENTRY, |_, ret, _| {
        super::dummy();
        ret[0] = rmi::SUCCESS;
    });

    listen!(mainloop, rmi::DATA_CREATE, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;
        // target_pa: location where realm data is created.
        let target_pa = arg[0];
        let rd = unsafe { Rd::into(arg[1]) };
        let ipa = arg[2];
        let src_pa = arg[3];
        let _flags = arg[4];

        let realm_id = rd.id();
        let granule_sz = 4096;

        // Make sure DATA_CREATE is only processed
        // when the realm is in its New state.
        if !rd.at_state(State::New) {
            ret[0] = rmi::RET_FAIL;
            return;
        }
        // 1. map src to rmm
        if granule::set_granule(target_pa, GranuleState::Data, mm) != granule::RET_SUCCESS {
            ret[0] = rmi::ERROR_INPUT;
            return;
        }
        mm.map(src_pa, false);

        // 3. copy src to _data
        unsafe {
            core::ptr::copy_nonoverlapping(src_pa as *const u8, target_pa as *mut u8, granule_sz);
        }

        // 4. map ipa to _taget_pa into S2 table
        let prot = rmi::MapProt::new(0);
        let res = rmi.map(realm_id, ipa, target_pa, granule_sz, prot.get());
        match res {
            Ok(_) => ret[0] = rmi::SUCCESS,
            Err(_) => ret[0] = rmi::RET_FAIL,
        }

        // TODO: 5. perform measure

        // 6. unmap src and _taget_pa from rmm
        mm.unmap(src_pa);
    });

    listen!(mainloop, rmi::DATA_CREATE_UNKNOWN, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;
        // target_phys: location where realm data is created.
        let target_pa = arg[0];
        let rd = unsafe { Rd::into(arg[1]) };
        let ipa = arg[2];

        let realm_id = rd.id();
        let granule_sz = 4096;

        // 0. Make sure granule state can make a transition to DATA
        if granule::set_granule(target_pa, GranuleState::Data, mm) != granule::RET_SUCCESS {
            ret[0] = rmi::ERROR_INPUT;
            warn!("DATA_CREATE_UNKNOWN: Unable to set granule state to DATA");
            return;
        }

        // 1. map ipa to target_pa into S2 table
        let prot = rmi::MapProt::new(0);
        let res = rmi.map(realm_id, ipa, target_pa, granule_sz, prot.get());
        match res {
            Ok(_) => ret[0] = rmi::SUCCESS,
            Err(_) => ret[0] = rmi::RET_FAIL,
        }

        // TODO: 2. perform measure
    });

    listen!(mainloop, rmi::DATA_DESTORY, |arg, ret, rmm| {
        let mm = rmm.mm;
        let target_data = arg[0];
        if granule::set_granule(target_data, GranuleState::Delegated, mm) != granule::RET_SUCCESS {
            ret[0] = rmi::ERROR_INPUT;
            return;
        }

        ret[0] = rmi::SUCCESS;
    });

    // Map an unprotected IPA to a non-secure PA.
    listen!(mainloop, rmi::RTT_MAP_UNPROTECTED, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let rd = unsafe { Rd::into(arg[0]) };
        let ipa = arg[1];
        let _level = arg[2];
        let ns_pa = arg[3];

        // islet stores rd as realm id
        let realm_id = rd.id();
        let granule_sz = 4096;
        let mut prot = rmi::MapProt(0);
        prot.set_bit(rmi::MapProt::NS_PAS);
        let _ret = rmi.map(realm_id, ipa, ns_pa, granule_sz, prot.get());
        ret[0] = rmi::SUCCESS;
    });
}
