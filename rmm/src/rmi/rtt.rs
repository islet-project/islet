extern crate alloc;

use super::realm::{rd::State, Rd};
use super::rec::Rec;
use crate::event::Mainloop;
use crate::granule::{is_not_in_realm, set_granule, GranuleState, GRANULE_SIZE};
use crate::host::pointer::Pointer as HostPointer;
use crate::host::DataPage;
use crate::listen;
use crate::rmi;
use crate::rmi::error::Error;
use crate::{get_granule, get_granule_if, set_state_and_get_granule};

pub const RTT_PAGE_LEVEL: usize = 3;

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
    listen!(mainloop, rmi::RTT_CREATE, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        let rtt_addr = arg[0];
        let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let ipa = arg[2];
        let level = arg[3];

        rmi.rtt_create(rd.id(), rtt_addr, ipa, level)?;
        Ok(())
    });

    listen!(mainloop, rmi::RTT_DESTROY, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        let rtt_addr = arg[0];
        let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let ipa = arg[2];
        let level = arg[3];

        rmi.rtt_destroy(rd, rtt_addr, ipa, level)?;
        Ok(())
    });

    listen!(mainloop, rmi::RTT_INIT_RIPAS, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let ipa = arg[1];
        let level = arg[2];

        if rd.state() != State::New {
            return Err(Error::RmiErrorInput);
        }
        rmi.rtt_init_ripas(rd.id(), ipa, level)?;
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
                return Err(Error::RmiErrorInput); //XXX: check this again
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
            rmi.make_shared(rd.id(), ipa, level)?;
        } else if ripas as u64 == RIPAS_RAM {
            rmi.make_exclusive(rd.id(), ipa, level)?;
        } else {
            unreachable!();
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
        let rd = arg[1];
        let ipa = arg[2];
        let src_pa = arg[3];
        let _flags = arg[4];

        if target_pa == rd || target_pa == src_pa || rd == src_pa {
            return Err(Error::RmiErrorInput);
        }

        // rd granule lock
        let rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let realm_id = rd.id();

        // Make sure DATA_CREATE is only processed
        // when the realm is in its New state.
        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm);
        }

        validate_ipa(ipa, rd.ipa_bits())?;

        if !is_not_in_realm(src_pa) {
            return Err(Error::RmiErrorInput);
        };

        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        let target_page = target_page_granule.content_mut::<DataPage>();
        rmm.page_table.map(target_pa, true);

        // read src page
        let src_page = copy_from_host_or_ret!(DataPage, src_pa);

        // 3. copy src to _data
        *target_page = src_page;

        // 4. map ipa to taget_pa in S2 table
        rmi.data_create(realm_id, ipa, target_pa)?;

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

        // 0. Make sure granule state can make a transition to DATA
        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        rmm.page_table.map(target_pa, true);

        // 1. map ipa to target_pa in S2 table
        rmi.data_create(realm_id, ipa, target_pa)?;

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

        let level = arg[2];
        let host_s2tte = arg[3];
        rmi.rtt_map_unprotected(rd, ipa, level, host_s2tte)?;
        Ok(())
    });

    // Unmap a non-secure PA at an unprotected IPA
    listen!(mainloop, rmi::RTT_UNMAP_UNPROTECTED, |arg, _ret, rmm| {
        let rmi = rmm.rmi;
        let ipa = arg[1];

        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let realm_id = rd.id();

        let level = arg[2];
        rmi.rtt_unmap_unprotected(realm_id, ipa, level)?;
        Ok(())
    });
}

fn realm_ipa_size(ipa_bits: usize) -> usize {
    1 << ipa_bits
}

pub fn realm_par_size(ipa_bits: usize) -> usize {
    realm_ipa_size(ipa_bits) / 2
}

fn validate_ipa(ipa: usize, ipa_bits: usize) -> Result<(), Error> {
    if ipa % GRANULE_SIZE != 0 {
        return Err(Error::RmiErrorInput);
    }

    if ipa >= realm_par_size(ipa_bits) {
        return Err(Error::RmiErrorInput);
    }

    Ok(())
}
