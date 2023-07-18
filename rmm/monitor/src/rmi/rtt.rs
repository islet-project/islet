use super::realm::{rd::State, Rd};
use super::rec::Rec;

use crate::event::Mainloop;
use crate::host::pointer::Pointer as HostPointer;
use crate::host::DataPage;
use crate::listen;
use crate::rmi;
use crate::rmi::error::Error;
use crate::rmm::granule;
use crate::rmm::granule::GranuleState;

extern crate alloc;

const RIPAS_EMPTY: u64 = 0;
const RIPAS_RAM: u64 = 1;

fn level_to_size(level: usize) -> u64 {
    // TODO: get the translation granule from src/armv9
    match level {
        0 => 512 << 30, // 512GB
        1 => 1 << 30,   // 1GB
        2 => 2 << 20,   // 2MB
        3 => 1 << 12,   // 4KB
        _ => 0,
    }
}

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::RTT_INIT_RIPAS, |_, _, _| {
        super::dummy();
        Ok(())
    });

    listen!(mainloop, rmi::RTT_SET_RIPAS, |arg, _ret, rmm| {
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
                return Err(Error::RmiErrorRtt);
            }
        }
        // TODO: update mapping
        super::dummy();
        let map_size = level_to_size(level);
        rec.inc_ripas_addr(map_size);
        Ok(())
    });

    listen!(mainloop, rmi::RTT_READ_ENTRY, |arg, ret, _| {
        super::dummy();

        // TODO: this code is a workaround to avoid kernel errors (host linux)
        //       once RTT_READ_ENTRY gets implemented properly, it should be removed.
        ret[1] = arg[2];
        Ok(())
    });

    listen!(mainloop, rmi::DATA_CREATE, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        let mm = rmm.mm;
        // target_pa: location where realm data is created.
        let target_pa = arg[0];
        let rd = unsafe { Rd::into(arg[1]) };
        let ipa = arg[2];
        let src_pa = arg[3];
        let _flags = arg[4];

        let realm_id = rd.id();

        // Make sure DATA_CREATE is only processed
        // when the realm is in its New state.
        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm);
        }
        // 1. map src to rmm
        if granule::set_granule(target_pa, GranuleState::Data, mm) != granule::RET_SUCCESS {
            return Err(Error::RmiErrorInput);
        }
        host_pointer_or_ret!(src_page, DataPage, src_pa, mm, ret[0]);

        // 3. copy src to _data
        unsafe {
            core::ptr::copy_nonoverlapping(
                src_page.as_ptr(),
                target_pa as *mut u8,
                core::mem::size_of::<DataPage>(),
            );
        }

        // 4. map ipa to _taget_pa into S2 table
        let prot = rmi::MapProt::new(0);
        let res = rmi.map(
            realm_id,
            ipa,
            target_pa,
            core::mem::size_of::<DataPage>(),
            prot.get(),
        );
        match res {
            Ok(_) => {}
            Err(val) => return Err(val),
        }

        // TODO: 5. perform measure
        Ok(())
    });

    listen!(mainloop, rmi::DATA_CREATE_UNKNOWN, |arg, _ret, rmm| {
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
            warn!("DATA_CREATE_UNKNOWN: Unable to set granule state to DATA");
            return Err(Error::RmiErrorInput);
        }

        // 1. map ipa to target_pa into S2 table
        let prot = rmi::MapProt::new(0);
        let res = rmi.map(realm_id, ipa, target_pa, granule_sz, prot.get());
        match res {
            Ok(_) => {}
            Err(val) => return Err(val),
        }

        // TODO: 2. perform measure
        Ok(())
    });

    listen!(mainloop, rmi::DATA_DESTORY, |arg, _ret, rmm| {
        let mm = rmm.mm;
        let target_data = arg[0];
        if granule::set_granule(target_data, GranuleState::Delegated, mm) != granule::RET_SUCCESS {
            return Err(Error::RmiErrorInput);
        }

        Ok(())
    });

    // Map an unprotected IPA to a non-secure PA.
    listen!(mainloop, rmi::RTT_MAP_UNPROTECTED, |arg, _ret, rmm| {
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
        Ok(())
    });
}
