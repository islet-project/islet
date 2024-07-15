extern crate alloc;

use crate::event::Mainloop;
use crate::granule::{
    is_granule_aligned, is_not_in_realm, set_granule, GranuleState, GRANULE_SIZE,
};
use crate::host;
use crate::host::DataPage;
use crate::listen;
use crate::measurement::HashContext;
use crate::realm::mm::rtt;
use crate::realm::mm::rtt::{RTT_MIN_BLOCK_LEVEL, RTT_PAGE_LEVEL};
use crate::realm::mm::stage2_tte::S2TTE;
use crate::realm::rd::{Rd, State};
use crate::rec::Rec;
use crate::rmi;
use crate::rmi::error::Error;
#[cfg(not(feature = "gst_page_table"))]
use crate::{get_granule, get_granule_if};
#[cfg(feature = "gst_page_table")]
use crate::{get_granule, get_granule_if, set_state_and_get_granule};

fn is_valid_rtt_cmd(ipa: usize, level: usize, ipa_bits: usize) -> bool {
    if level > RTT_PAGE_LEVEL {
        return false;
    }

    if ipa >= realm_ipa_size(ipa_bits) {
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

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::RTT_CREATE, |arg, _ret, _rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rtt_addr = arg[1];
        let rd = rd_granule.content::<Rd>()?;
        let ipa = arg[2];
        let level = arg[3];

        let min_level = rd.s2_starting_level() as usize + 1;

        if (level < min_level)
            || (level > RTT_PAGE_LEVEL)
            || !is_valid_rtt_cmd(ipa, level - 1, rd.ipa_bits())
        {
            return Err(Error::RmiErrorInput);
        }
        if rtt_addr == arg[0] {
            return Err(Error::RmiErrorInput);
        }
        rtt::create(&rd, rtt_addr, ipa, level)?;
        Ok(())
    });

    listen!(mainloop, rmi::RTT_DESTROY, |arg, ret, _rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let ipa = arg[1];
        let level = arg[2];

        let min_level = rd.s2_starting_level() as usize + 1;

        if (level < min_level)
            || (level > RTT_PAGE_LEVEL)
            || !is_valid_rtt_cmd(ipa, level - 1, rd.ipa_bits())
        {
            return Err(Error::RmiErrorInput);
        }
        let (ipa, walk_top) = rtt::destroy(&rd, ipa, level, |t| {
            ret[2] = t;
        })?;
        ret[1] = ipa;
        ret[2] = walk_top;
        Ok(())
    });

    listen!(mainloop, rmi::RTT_INIT_RIPAS, |arg, ret, _rmm| {
        let mut rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;
        let base = arg[1];
        let top = arg[2];

        if rd.state() != State::New {
            return Err(Error::RmiErrorRealm(0));
        }

        if top <= base {
            return Err(Error::RmiErrorInput);
        }

        if !is_valid_rtt_cmd(base, RTT_PAGE_LEVEL, rd.ipa_bits())
            || !is_valid_rtt_cmd(top, RTT_PAGE_LEVEL, rd.ipa_bits())
            || !is_protected_ipa(base, rd.ipa_bits())
            || !is_protected_ipa(top - GRANULE_SIZE, rd.ipa_bits())
        {
            return Err(Error::RmiErrorInput);
        }

        let out_top = rtt::init_ripas(&rd, base, top)?;
        ret[1] = out_top; //This is walk_top

        //TODO: Update the function according to the changes from eac5
        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(&mut rd)?.measure_ripas_granule(base, RTT_PAGE_LEVEL as u8)?;
        Ok(())
    });

    listen!(mainloop, rmi::RTT_SET_RIPAS, |arg, ret, _rmm| {
        let base = arg[2];
        let top = arg[3];

        if arg[0] == arg[1] {
            warn!("Granules of RD and REC shouldn't be identical");
            return Err(Error::RmiErrorInput);
        }
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let mut rec_granule = get_granule_if!(arg[1], GranuleState::Rec)?;
        let mut rec = rec_granule.content_mut::<Rec<'_>>()?;
        if rec.realmid()? != rd.id() {
            warn!("RD:{:X} doesn't own REC:{:X}", arg[0], arg[1]);
            return Err(Error::RmiErrorRec);
        }

        if rec.ripas_addr() != base as u64 || rec.ripas_end() < top as u64 {
            return Err(Error::RmiErrorInput);
        }

        if !is_granule_aligned(base)
            || !is_granule_aligned(top)
            || !is_protected_ipa(base, rd.ipa_bits())
            || !is_protected_ipa(top - GRANULE_SIZE, rd.ipa_bits())
        {
            return Err(Error::RmiErrorInput);
        }

        let out_top = rtt::set_ripas(&rd, base, top, rec.ripas_state(), rec.ripas_flags())?;
        ret[1] = out_top;
        rec.set_ripas_addr(out_top as u64);
        Ok(())
    });

    listen!(mainloop, rmi::RTT_READ_ENTRY, |arg, ret, _rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let ipa = arg[1];
        let level = arg[2];
        if !is_valid_rtt_cmd(ipa, level, rd.ipa_bits()) {
            return Err(Error::RmiErrorInput);
        }

        let res = rtt::read_entry(&rd, ipa, level)?;
        ret[1..5].copy_from_slice(&res[0..4]);

        Ok(())
    });

    listen!(mainloop, rmi::DATA_CREATE, |arg, _ret, rmm| {
        // target_pa: location where realm data is created.
        let rd = arg[0];
        let target_pa = arg[1];
        let ipa = arg[2];
        let src_pa = arg[3];
        let flags = arg[4];

        if target_pa == rd || target_pa == src_pa || rd == src_pa {
            return Err(Error::RmiErrorInput);
        }

        // rd granule lock
        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;

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
        let mut target_page = target_page_granule.content_mut::<DataPage>()?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(target_pa, true);

        // copy src to target
        host::copy_to_obj::<DataPage>(src_pa, &mut target_page).ok_or(Error::RmiErrorInput)?;

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(&mut rd)?.measure_data_granule(&target_page, ipa, flags)?;

        // map ipa to taget_pa in S2 table
        rtt::data_create(&rd, ipa, target_pa, false)?;

        set_granule(&mut target_page_granule, GranuleState::Data)?;
        Ok(())
    });

    listen!(mainloop, rmi::DATA_CREATE_UNKNOWN, |arg, _ret, rmm| {
        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;

        // target_phys: location where realm data is created.
        let target_pa = arg[1];
        let ipa = arg[2];
        if target_pa == arg[0] {
            return Err(Error::RmiErrorInput);
        }

        validate_ipa(ipa, rd.ipa_bits())?;

        // 0. Make sure granule state can make a transition to DATA
        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(target_pa, true);

        // 1. map ipa to target_pa in S2 table
        rtt::data_create(&rd, ipa, target_pa, true)?;

        // TODO: 2. perform measure
        // L0czek - not needed here see: tf-rmm/runtime/rmi/rtt.c:883
        set_granule(&mut target_page_granule, GranuleState::Data)?;
        Ok(())
    });

    listen!(mainloop, rmi::DATA_DESTROY, |arg, ret, _rmm| {
        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let ipa = arg[1];

        if !is_protected_ipa(ipa, rd.ipa_bits())
            || !is_valid_rtt_cmd(ipa, RTT_PAGE_LEVEL, rd.ipa_bits())
        {
            return Err(Error::RmiErrorInput);
        }

        let (pa, top) = rtt::data_destroy(&rd, ipa, |t| {
            ret[2] = t;
        })?;

        // data granule lock and change state
        #[cfg(feature = "gst_page_table")]
        set_state_and_get_granule!(pa, GranuleState::Delegated)?;

        #[cfg(not(feature = "gst_page_table"))]
        {
            let mut granule = get_granule!(pa)?;
            set_granule(&mut granule, GranuleState::Delegated)?;
        }

        ret[1] = pa;
        ret[2] = top;
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
        let rd = rd_granule.content::<Rd>()?;

        if !is_valid_rtt_cmd(ipa, level, rd.ipa_bits()) {
            return Err(Error::RmiErrorInput);
        }
        rtt::map_unprotected(&rd, ipa, level, host_s2tte)?;
        Ok(())
    });

    // Unmap a non-secure PA at an unprotected IPA
    listen!(mainloop, rmi::RTT_UNMAP_UNPROTECTED, |arg, ret, _rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;

        let ipa = arg[1];

        let level = arg[2];
        if (level < RTT_MIN_BLOCK_LEVEL)
            || (level > RTT_PAGE_LEVEL)
            || !is_valid_rtt_cmd(ipa, level, rd.ipa_bits())
        {
            return Err(Error::RmiErrorInput);
        }

        let top = rtt::unmap_unprotected(&rd, ipa, level, |t| {
            ret[1] = t;
        })?;
        ret[1] = top;

        Ok(())
    });
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

    if !is_valid_rtt_cmd(ipa, RTT_PAGE_LEVEL, ipa_bits) {
        return Err(Error::RmiErrorInput);
    }

    Ok(())
}
