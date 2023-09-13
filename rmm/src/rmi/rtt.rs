extern crate alloc;

use super::realm::{rd::State, Rd};
use super::rec::Rec;
use crate::event::Mainloop;
use crate::granule::{set_granule, GranuleState, GRANULE_SIZE};
use crate::host::pointer::Pointer as HostPointer;
use crate::host::DataPage;
use crate::listen;
use crate::mm::page_table;
use crate::rmi;
use crate::rmi::error::Error;
use crate::{get_granule, get_granule_if, set_state_and_get_granule};

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
        let rmi = rmm.rmi;
        let ipa = arg[2];
        let level = arg[3];
        let ripas = arg[4];

        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let mut rec_granule = get_granule_if!(arg[1], GranuleState::Rec)?;
        let rec = rec_granule.content_mut::<Rec>();

        let mut prot = rmi::MapProt::new(0);
        match ripas as u64 {
            RIPAS_EMPTY => prot.set_bit(rmi::MapProt::NS_PAS),
            RIPAS_RAM => { /* do nothing: ripas ram by default */ }
            _ => {
                warn!("Unknown RIPAS:{}", ripas);
                return Err(Error::RmiErrorRtt);
            }
        }

        if rec.ripas_state() != ripas as u8 {
            return Err(Error::RmiErrorInput);
        }

        if rec.ripas_addr() != ipa as u64 {
            return Err(Error::RmiErrorInput);
        }

        let map_size = level_to_size(level);
        if ipa as u64 + map_size > rec.ripas_end() {
            return Err(Error::RmiErrorInput);
        }

        if ripas as u64 == RIPAS_EMPTY {
            let ret = rmi.make_shared(rd.id(), ipa, level);
            if ret.is_err() {
                return Err(Error::RmiErrorInput);
            }
        }
        rec.inc_ripas_addr(map_size);
        Ok(())
    });

    listen!(mainloop, rmi::RTT_READ_ENTRY, |arg, ret, rmm| {
        let rmi = rmm.rmi;
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let ipa = arg[1];
        let level = arg[2];

        let res = rmi.rtt_read_entry(rd.id(), ipa, level)?;
        ret[1..5].copy_from_slice(&res[0..4]);

        Ok(())
    });

    listen!(mainloop, rmi::DATA_CREATE, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        // target_pa: location where realm data is created.
        let target_pa = arg[0];
        let ipa = arg[2];
        let src_pa = arg[3];
        let _flags = arg[4];

        // rd granule lock
        let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let realm_id = rd.id();

        // Make sure DATA_CREATE is only processed
        // when the realm is in its New state.
        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm);
        }

        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        let target_page = target_page_granule.content_mut::<DataPage>();
        page_table::map(target_pa, true);

        // read src page
        let src_page = copy_from_host_or_ret!(DataPage, src_pa);

        // 3. copy src to _data
        *target_page = src_page;

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
        set_granule(&mut target_page_granule, GranuleState::Data)?;
        Ok(())
    });

    listen!(mainloop, rmi::DATA_CREATE_UNKNOWN, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        // target_phys: location where realm data is created.
        let target_pa = arg[0];
        let ipa = arg[2];

        // rd granule lock
        let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let realm_id = rd.id();
        let granule_sz = GRANULE_SIZE;

        // 0. Make sure granule state can make a transition to DATA
        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        page_table::map(target_pa, true);

        // 1. map ipa to target_pa into S2 table
        let prot = rmi::MapProt::new(0);
        let res = rmi.map(realm_id, ipa, target_pa, granule_sz, prot.get());
        match res {
            Ok(_) => {}
            Err(val) => return Err(val),
        }

        // TODO: 2. perform measure
        set_granule(&mut target_page_granule, GranuleState::Data)?;
        Ok(())
    });

    listen!(mainloop, rmi::DATA_DESTROY, |arg, _ret, rmm| {
        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let realm_id = rd.id();
        let ipa = arg[1];

        let pa = rmm.rmi.data_destroy(realm_id, ipa)?;

        // data granule lock and change state
        set_state_and_get_granule!(pa, GranuleState::Delegated)?;
        Ok(())
    });

    // Map an unprotected IPA to a non-secure PA.
    listen!(mainloop, rmi::RTT_MAP_UNPROTECTED, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        let ipa = arg[1];

        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let realm_id = rd.id();

        let level = arg[2];
        let host_s2tte = arg[3];
        rmi.rtt_map_unprotected(realm_id, ipa, level, host_s2tte)?;
        Ok(())
    });

    // Unmap a non-secure PA at an unprotected IPA
    listen!(mainloop, rmi::RTT_UNMAP_UNPROTECTED, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        let ipa = arg[1];

        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        rmi.unmap(rd.id(), ipa, GRANULE_SIZE)?;
        Ok(())
    });
}
