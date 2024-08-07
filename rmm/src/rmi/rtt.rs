extern crate alloc;

use super::realm::{rd::State, Rd};
use super::rec::Rec;
use crate::event::Mainloop;
use crate::granule::{
    is_granule_aligned, is_not_in_realm, set_granule, GranuleState, GRANULE_SHIFT, GRANULE_SIZE,
};
use crate::host;
use crate::host::DataPage;
use crate::listen;
use crate::measurement::HashContext;
use crate::realm::mm::stage2_tte::{invalid_ripas, S2TTE};
use crate::rmi;
use crate::rmi::error::Error;
#[cfg(not(feature = "gst_page_table"))]
use crate::{get_granule, get_granule_if};
#[cfg(feature = "gst_page_table")]
use crate::{get_granule, get_granule_if, set_state_and_get_granule};

pub const RTT_MIN_BLOCK_LEVEL: usize = 2;
pub const RTT_PAGE_LEVEL: usize = 3;
pub const S2TTE_STRIDE: usize = GRANULE_SHIFT - 3;

const RIPAS_EMPTY: u64 = 0;
const RIPAS_RAM: u64 = 1;
const RIPAS_SHARED: u64 = 3;

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

fn is_valid_rtt_cmd(ipa: usize, level: usize) -> bool {
    if level > RTT_PAGE_LEVEL {
        return false;
    }
    let mask = match level {
        0 => S2TTE::ADDR_L0_PAGE,
        1 => S2TTE::ADDR_L1_PAGE,
        2 => S2TTE::ADDR_L2_PAGE,
        3 => S2TTE::ADDR_L3_PAGE,
        _ => unreachable!(),
    };
    if ipa & mask as usize != ipa {
        return false;
    }
    true
}

fn unmap_shared_realm_memory(
    arg: &[usize],
    _ret: &mut [usize],
    _rmm: &crate::Monitor,
) -> core::result::Result<(), Error> {
    // target_phys: location where realm data is created.
    let target_pa = arg[0];
    let ipa = arg[2];

    warn!(
        "RMI::UNMAP_SHARED_REALM_MEM called with target_pa: {:x}, ipa: {:x}",
        target_pa, ipa
    );

    if target_pa == arg[1] {
        return Err(Error::RmiErrorInput);
    }

    // rd granule lock
    let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
    let rd = rd_granule.content::<Rd>();

    validate_ipa(ipa, rd.ipa_bits())?;

    warn!("check the target_pa is undelegated granule");
    // First of all, the memory must be destroyed on the owner realm side
    let _ = get_granule_if!(target_pa, GranuleState::Undelegated)?;

    warn!("call rtt::unmap_shared_realm_memory");
    let ret = crate::rtt::unmap_shared_realm_memory(rd, ipa, target_pa);
    if let Err(e) = &ret {
        warn!("rtt::unmap_shared_realm_memory return error: {:?}", e);
        return ret;
    }

    warn!("RMI::UNMAP_SHARED_REALM_MEM done!");
    Ok(())
}

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::RTT_CREATE, |arg, _ret, _rmm| {
        let rtt_addr = arg[0];
        let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let ipa = arg[2];
        let level = arg[3];

        if !is_valid_rtt_cmd(ipa, level) {
            return Err(Error::RmiErrorInput);
        }
        if rtt_addr == arg[1] {
            return Err(Error::RmiErrorInput);
        }
        crate::rtt::create(rd, rtt_addr, ipa, level)?;
        Ok(())
    });

    listen!(mainloop, rmi::RTT_DESTROY, |arg, _ret, _rmm| {
        let rtt_addr = arg[0];
        let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let ipa = arg[2];
        let level = arg[3];

        if !is_valid_rtt_cmd(ipa, level) {
            return Err(Error::RmiErrorInput);
        }
        crate::rtt::destroy(rd, rtt_addr, ipa, level)?;
        Ok(())
    });

    listen!(mainloop, rmi::RTT_INIT_RIPAS, |arg, _ret, _rmm| {
        let mut rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content_mut::<Rd>();
        let ipa = arg[1];
        let level = arg[2];
        let shared = arg[3];
        let mut ripas = invalid_ripas::RAM;

        if rd.state() != State::New {
            return Err(Error::RmiErrorRealm(0));
        }
        if !is_valid_rtt_cmd(ipa, level) {
            return Err(Error::RmiErrorInput);
        }

        if shared != 0 {
            ripas = invalid_ripas::SHARED;
        }

        crate::rtt::init_ripas(rd, ipa, level, ripas)?;

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(rd)?.measure_ripas_granule(ipa, level as u8)?;

        Ok(())
    });

    listen!(mainloop, rmi::RTT_SET_RIPAS, |arg, _ret, _rmm| {
        let ipa = arg[2];
        let level = arg[3];
        let ripas = arg[4];

        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let mut rec_granule = get_granule_if!(arg[1], GranuleState::Rec)?;
        let rec = rec_granule.content_mut::<Rec<'_>>();

        trace!(
            "RTT_SET_RIPAS start: ipa {:x}, level {}, ripas {}, rd.id(): {}",
            ipa,
            level,
            ripas,
            rd.id()
        );

        let mut prot = rmi::MapProt::new(0);
        match ripas as u64 {
            RIPAS_EMPTY => prot.set_bit(rmi::MapProt::NS_PAS),
            RIPAS_RAM => { /* do nothing: ripas ram by default */ }
            RIPAS_SHARED => {
                /* do nothing */
                warn!("ripas shared is called! ipa {:x}", ipa);
            }
            _ => {
                warn!("Unknown RIPAS:{}", ripas);
                return Err(Error::RmiErrorInput); //XXX: check this again
            }
        }

        if rec.ripas_state() != ripas as u8 {
            warn!(
                "ripas_state() {} != ripas {}, ipa: {:x}",
                rec.ripas_state(),
                ripas,
                ipa
            );
            return Err(Error::RmiErrorInput);
        }

        if rec.ripas_addr() != ipa as u64 {
            warn!("ripas_addr() {:x} != ipa {:x}", rec.ripas_addr(), ipa);
            return Err(Error::RmiErrorInput);
        }

        let map_size = level_to_size(level);
        if ipa as u64 + map_size > rec.ripas_end() {
            warn!(
                "ipa + map_size () {:x} > rec.ripas_end(){:x}",
                ipa as u64 + map_size,
                rec.ripas_end()
            );
            return Err(Error::RmiErrorInput);
        }

        if ripas as u64 == RIPAS_EMPTY {
            crate::rtt::make_shared(rd, ipa, level)?;
        } else if ripas as u64 == RIPAS_RAM {
            crate::rtt::make_exclusive(rd, ipa, level)?;
        } else if ripas as u64 == RIPAS_SHARED {
            let ret = crate::rtt::make_shared_ripas(rd, ipa, level);
            if let Err(e) = &ret {
                warn!("make_shared_ripas failed with {:?}", e);
                return ret;
            }
        } else {
            unreachable!();
        }
        rec.inc_ripas_addr(map_size);

        trace!(
            "RTT_SET_RIPAS end: ipa {:x}, level {}, ripas {}",
            ipa,
            level,
            ripas
        );
        Ok(())
    });

    listen!(mainloop, rmi::RTT_READ_ENTRY, |arg, ret, _rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let ipa = arg[1];
        let level = arg[2];
        if !is_valid_rtt_cmd(ipa, level) {
            return Err(Error::RmiErrorInput);
        }

        let res = crate::rtt::read_entry(rd, ipa, level)?;
        ret[1..5].copy_from_slice(&res[0..4]);

        Ok(())
    });

    listen!(mainloop, rmi::DATA_CREATE, |arg, _ret, rmm| {
        // target_pa: location where realm data is created.
        let target_pa = arg[0];
        let rd = arg[1];
        let ipa = arg[2];
        let src_pa = arg[3];
        let flags = arg[4];

        if target_pa == rd || target_pa == src_pa || rd == src_pa {
            return Err(Error::RmiErrorInput);
        }

        // rd granule lock
        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let rd = rd_granule.content_mut::<Rd>();

        // Make sure DATA_CREATE is only processed
        // when the realm is in its New state.
        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm(0));
        }

        validate_ipa(ipa, rd.ipa_bits())?;

        if !is_not_in_realm(src_pa) {
            return Err(Error::RmiErrorInput);
        };

        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        let target_page = target_page_granule.content_mut::<DataPage>();
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(target_pa, true);

        // read src page
        let src_page = host::copy_from::<DataPage>(src_pa).ok_or(Error::RmiErrorInput)?;

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(rd)?.measure_data_granule(&src_page, ipa, flags)?;

        // 3. copy src to _data
        *target_page = src_page;

        // 4. map ipa to taget_pa in S2 table
        crate::rtt::data_create(rd, ipa, target_pa)?;

        set_granule(&mut target_page_granule, GranuleState::Data)?;
        Ok(())
    });

    listen!(mainloop, rmi::DATA_CREATE_UNKNOWN, |arg, _ret, rmm| {
        // target_phys: location where realm data is created.
        let target_pa = arg[0];
        let ipa = arg[2];
        if target_pa == arg[1] {
            return Err(Error::RmiErrorInput);
        }

        // rd granule lock
        let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();

        validate_ipa(ipa, rd.ipa_bits())?;

        // 0. Make sure granule state can make a transition to DATA
        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(target_pa, true);

        // 1. map ipa to target_pa in S2 table
        crate::rtt::data_create(rd, ipa, target_pa)?;

        // TODO: 2. perform measure
        // L0czek - not needed here see: tf-rmm/runtime/rmi/rtt.c:883
        set_granule(&mut target_page_granule, GranuleState::Data)?;
        Ok(())
    });

    listen!(mainloop, rmi::SHARED_DATA_CREATE, |arg, _ret, rmm| {
        // target_phys: location where realm data is created.
        let target_pa = arg[0];
        let ipa = arg[2];
        if target_pa == arg[1] {
            return Err(Error::RmiErrorInput);
        }

        warn!(
            "shared_data_create is called. pa {:x}, ipa {:x}",
            target_pa, ipa
        );

        // rd granule lock
        let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();

        validate_ipa(ipa, rd.ipa_bits())?;

        // 0. Make sure granule state can make a transition to DATA
        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(target_pa, true);

        // 1. map ipa to target_pa in S2 table
        crate::rtt::shared_data_create(rd, ipa, target_pa)?;

        // TODO: 2. perform measure
        // L0czek - not needed here see: tf-rmm/runtime/rmi/rtt.c:883
        set_granule(&mut target_page_granule, GranuleState::SharedData)?;
        target_page_granule.inc_ref()?;

        warn!("shared_data_create done. pa {:x}, ipa {:x}", target_pa, ipa);
        Ok(())
    });

    listen!(mainloop, rmi::DATA_DESTROY, |arg, _ret, _rmm| {
        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let ipa = arg[1];

        let pa = crate::rtt::data_destroy(rd, ipa)?;

        // data granule lock and change state
        #[cfg(feature = "gst_page_table")]
        set_state_and_get_granule!(pa, GranuleState::Delegated)?;
        #[cfg(not(feature = "gst_page_table"))]
        {
            let mut granule = get_granule!(pa)?;
            set_granule(&mut granule, GranuleState::Delegated)?;
        }
        Ok(())
    });

    listen!(mainloop, rmi::SHARED_DATA_DESTROY, |arg, ret, _rmm| {
        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();
        let ipa = arg[1];

        warn!("SHARED_DATA_DESTROY start. ipa {:x}", ipa);

        let pa = crate::rtt::shared_data_destroy(rd, ipa)?;
        let mut granule = get_granule_if!(pa, GranuleState::SharedData)?;

        granule.dec_ref()?;

        ret[1] = granule.get_ref() as usize;
        warn!("SHARED_DATA_DESTROY: get_ref: {:x}", ret[1]);

        if granule.get_ref() == 0 {
            set_granule(&mut granule, GranuleState::Delegated)?;
        }

        Ok(())
    });

    // Map an unprotected IPA to a non-secure PA.
    listen!(mainloop, rmi::RTT_MAP_UNPROTECTED, |arg, _ret, _rmm| {
        let ipa = arg[1];
        let level = arg[2];
        let host_s2tte = arg[3];
        let s2tte = S2TTE::from(host_s2tte);
        if !s2tte.is_host_ns_valid(level) {
            return Err(Error::RmiErrorInput);
        }

        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();

        if !is_valid_rtt_cmd(ipa, level) {
            return Err(Error::RmiErrorInput);
        }
        crate::rtt::map_unprotected(rd, ipa, level, host_s2tte)?;
        Ok(())
    });

    // Unmap a non-secure PA at an unprotected IPA
    listen!(mainloop, rmi::RTT_UNMAP_UNPROTECTED, |arg, _ret, _rmm| {
        let ipa = arg[1];

        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();

        let level = arg[2];
        if !is_valid_rtt_cmd(ipa, level) {
            return Err(Error::RmiErrorInput);
        }
        crate::rtt::unmap_unprotected(rd, ipa, level)?;
        Ok(())
    });

    listen!(mainloop, rmi::MAP_SHARED_MEM_AS_RO, |arg, _ret, rmm| {
        // target_phys: location where realm data is created.
        let target_pa = arg[0];
        let ipa = arg[2];
        let size = arg[3];
        if target_pa == arg[1] {
            return Err(Error::RmiErrorInput);
        }

        if size == 0 {
            return unmap_shared_realm_memory(arg, _ret, rmm);
        }

        warn!(
            "RMI::MAP_SHARED_MEM_AS_RO called with target_pa: {:x}, ipa: {:x}",
            target_pa, ipa
        );

        // rd granule lock
        let rd_granule = get_granule_if!(arg[1], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>();

        validate_ipa(ipa, rd.ipa_bits())?;

        warn!("check the target_pa is SharedData granule");
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::SharedData)?;

        warn!("call rtt::map_shared_mem_as_ro");
        let ret = crate::rtt::make_shared_as_readonly(rd, ipa, target_pa);
        if let Err(e) = &ret {
            warn!("rtt::map_shared_mem_as_ro return error: {:?}", e);
            return ret;
        }
        target_page_granule.inc_ref()?;

        warn!("RMI::MAP_SHARED_MEM_AS_RO done!");
        Ok(())
    });

    listen!(
        mainloop,
        rmi::UNMAP_SHARED_REALM_MEM,
        unmap_shared_realm_memory
    );
}

fn realm_ipa_size(ipa_bits: usize) -> usize {
    1 << ipa_bits
}

pub fn realm_par_size(ipa_bits: usize) -> usize {
    realm_ipa_size(ipa_bits) / 2
}

pub fn is_protected_ipa(ipa: usize, ipa_bits: usize) -> bool {
    ipa < realm_par_size(ipa_bits)
}

pub fn validate_ipa(ipa: usize, ipa_bits: usize) -> Result<(), Error> {
    if !is_granule_aligned(ipa) {
        error!("ipa: {:x} is not aligned with {:x}", ipa, GRANULE_SIZE);
        return Err(Error::RmiErrorInput);
    }

    if !is_protected_ipa(ipa, ipa_bits) {
        error!(
            "ipa: {:x} is not in protected ipa range {:x}",
            ipa,
            realm_par_size(ipa_bits)
        );
        return Err(Error::RmiErrorInput);
    }

    Ok(())
}
