use crate::granule::GRANULE_SIZE;
use crate::granule::{set_granule, GranuleState};
use crate::mm::translation::PageTable;
use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::page_table::pte::attribute;
use crate::realm::mm::page_table::pte::{permission, shareable};
use crate::realm::mm::stage2_tte::{desc_type, invalid_hipas, invalid_ripas};
use crate::realm::mm::stage2_tte::{RttPage, INVALID_UNPROTECTED, S2TTE};
use crate::realm::registry::get_realm;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;
use crate::rmi::realm::Rd;
use crate::rmi::rtt::RTT_PAGE_LEVEL;
use crate::rmi::rtt::S2TTE_STRIDE;
use crate::rmi::rtt_entry_state;
use crate::{get_granule, get_granule_if};
use armv9a::bits_in_reg;

pub fn create(id: usize, rtt_addr: usize, ipa: usize, level: usize) -> Result<(), Error> {
    let mut rtt_granule = get_granule_if!(rtt_addr, GranuleState::Delegated)?;
    let s2tt = rtt_granule.content_mut::<RttPage>();

    let (parent_s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level - 1, Error::RmiErrorInput)?;

    if last_level != level - 1 {
        return Err(Error::RmiErrorRtt(last_level));
    }

    let s2tt_len = s2tt.len();
    if parent_s2tte.is_unassigned() {
        if parent_s2tte.is_invalid_ripas() {
            panic!("invalid ripas");
        }
        let ripas = parent_s2tte.get_ripas();
        let mut new_s2tte = bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
        if ripas == invalid_ripas::EMPTY {
            new_s2tte |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
        } else if ripas == invalid_ripas::RAM {
            new_s2tte |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::RAM);
        } else {
            panic!("Unexpected ripas:{}", ripas);
        }

        for i in 0..s2tt_len {
            if let Some(elem) = s2tt.get_mut(i) {
                *elem = new_s2tte;
            }
        }
    } else if parent_s2tte.is_destroyed() {
        let new_s2tte = bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::DESTROYED);
        for i in 0..s2tt_len {
            if let Some(elem) = s2tt.get_mut(i) {
                *elem = new_s2tte;
            }
        }
    } else if parent_s2tte.is_assigned() {
        let mut pa: usize = parent_s2tte
            .address(level - 1)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
        let map_size = match level {
            3 => GRANULE_SIZE, // 4096
            2 => GRANULE_SIZE << S2TTE_STRIDE,
            1 => GRANULE_SIZE << (S2TTE_STRIDE * 2),
            0 => GRANULE_SIZE << (S2TTE_STRIDE * 3),
            _ => unreachable!(),
        };
        let mut flags = bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
        flags |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
        let mut new_s2tte = pa as u64 | flags;
        for i in 0..s2tt_len {
            if let Some(elem) = s2tt.get_mut(i) {
                *elem = new_s2tte;
            }
            pa += map_size;
            new_s2tte = pa as u64 | flags;
        }
    } else if parent_s2tte.is_valid(level - 1, false) {
        unimplemented!();
    } else if parent_s2tte.is_valid(level - 1, true) {
        unimplemented!();
    } else if parent_s2tte.is_table(level - 1) {
        return Err(Error::RmiErrorRtt(level - 1));
    } else {
        panic!("Unexpected s2tte value:{:X}", parent_s2tte.get());
    }

    set_granule(&mut rtt_granule, GranuleState::RTT)?;

    let parent_s2tte = rtt_addr as u64 | bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_TABLE);
    get_realm(id)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level - 1, parent_s2tte)?;

    // The below is added to avoid a fault regarding the RTT entry
    PageTable::get_ref().map(rtt_addr, true);

    Ok(())
}

pub fn destroy(rd: &Rd, rtt_addr: usize, ipa: usize, level: usize) -> Result<(), Error> {
    let id = rd.id();
    let (parent_s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level - 1, Error::RmiErrorRtt(0))?;

    if last_level != level - 1 {
        return Err(Error::RmiErrorRtt(last_level));
    }

    if !parent_s2tte.is_table(level - 1) {
        return Err(Error::RmiErrorRtt(level - 1));
    }

    let pa_table = parent_s2tte
        .address(RTT_PAGE_LEVEL)
        .ok_or(Error::RmiErrorInput)?
        .try_into()
        .or(Err(Error::RmiErrorInput))?;
    if rtt_addr != pa_table {
        return Err(Error::RmiErrorInput);
    }

    let mut g_rtt = get_granule_if!(rtt_addr, GranuleState::RTT)?;

    let parent_s2tte = if rd.addr_in_par(ipa) {
        bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::DESTROYED)
    } else {
        INVALID_UNPROTECTED
    };

    get_realm(id)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level - 1, parent_s2tte)?;

    set_granule(&mut g_rtt, GranuleState::Delegated)?;
    Ok(())
}

pub fn init_ripas(id: usize, ipa: usize, level: usize) -> Result<(), Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level));
    }

    if s2tte.is_table(level) || !s2tte.is_unassigned() {
        return Err(Error::RmiErrorRtt(level));
    }

    let mut new_s2tte = s2tte.get();
    new_s2tte |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::RAM);

    get_realm(id)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;

    Ok(())
}

pub fn get_ripas(id: usize, ipa: usize, level: usize) -> Result<u64, Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level));
    }

    if s2tte.is_destroyed() {
        error!("The s2tte is destroyed: {:x}", s2tte.get());
        return Err(Error::RmiErrorRtt(last_level));
    }
    Ok(s2tte.get_ripas())
}

pub fn read_entry(id: usize, ipa: usize, level: usize) -> Result<[usize; 4], Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level, Error::RmiErrorRtt(0))?;

    let r1 = last_level;
    let (mut r2, mut r3, mut r4) = (0, 0, 0);

    if s2tte.is_unassigned() {
        let ripas = s2tte.get_masked_value(S2TTE::INVALID_RIPAS);
        r2 = rtt_entry_state::RMI_UNASSIGNED;
        r4 = ripas as usize;
    } else if s2tte.is_destroyed() {
        r2 = rtt_entry_state::RMI_DESTROYED;
    } else if s2tte.is_assigned() {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        r3 = s2tte
            .address(last_level)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
        r4 = invalid_ripas::EMPTY as usize;
    } else if s2tte.is_valid(last_level, false) {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        r3 = s2tte
            .address(last_level)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
        r4 = invalid_ripas::RAM as usize;
    } else if s2tte.is_valid(last_level, true) {
        r2 = rtt_entry_state::RMI_VALID_NS;
        let addr_mask = match level {
            1 => S2TTE::ADDR_L1_PAGE,
            2 => S2TTE::ADDR_L2_PAGE,
            3 => S2TTE::ADDR_L3_PAGE,
            _ => {
                return Err(Error::RmiErrorRtt(0)); //XXX: check this again
            }
        };
        let mask = addr_mask | S2TTE::MEMATTR | S2TTE::AP | S2TTE::SH;
        r3 = (s2tte.get() & mask) as usize;
    } else if s2tte.is_table(last_level) {
        r2 = rtt_entry_state::RMI_TABLE;
        r3 = s2tte
            .address(RTT_PAGE_LEVEL)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
    } else {
        error!("Unexpected S2TTE value retrieved!");
    }
    Ok([r1, r2, r3, r4])
}

pub fn map_unprotected(rd: &Rd, ipa: usize, level: usize, host_s2tte: usize) -> Result<(), Error> {
    if rd.addr_in_par(ipa) {
        return Err(Error::RmiErrorInput);
    }

    let id = rd.id();
    let (s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level));
    }

    if !s2tte.is_unassigned() {
        return Err(Error::RmiErrorRtt(level));
    }

    let mut new_s2tte = host_s2tte as u64;
    if level == RTT_PAGE_LEVEL {
        new_s2tte |= bits_in_reg(S2TTE::NS, 1)
            | bits_in_reg(S2TTE::XN, 1)
            | bits_in_reg(S2TTE::AF, 1)
            | bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
    } else {
        new_s2tte |= bits_in_reg(S2TTE::NS, 1)
            | bits_in_reg(S2TTE::XN, 1)
            | bits_in_reg(S2TTE::AF, 1)
            | bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_BLOCK);
    }

    get_realm(id)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;

    Ok(())
}

pub fn unmap_unprotected(id: usize, ipa: usize, level: usize) -> Result<(), Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level));
    }

    if !s2tte.is_valid(level, true) {
        return Err(Error::RmiErrorRtt(level));
    }

    let new_s2tte: u64 = INVALID_UNPROTECTED;

    get_realm(id)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;

    //TODO: add page/block invalidation

    Ok(())
}

pub fn make_shared(id: usize, ipa: usize, level: usize) -> Result<(), Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level)); //XXX: check this again
    }

    // Reference (tf-rmm)      : smc_rtt_set_ripas() in runtime/rmi/rtt.c
    //           (realm-linux) : __set_memory_encrypted() in arch/arm64/mm/pageattr.c
    //           (nw-linux)    : set_ipa_state() and kvm_realm_unmap_range() in arch/arm64/kvm/rme.c
    //           (rmm-spec)    : Figure D2.1 Realm shared memory protocol flow
    if s2tte.is_valid(level, false) {
        // the case for ipa's range 0x8840_0000 - in realm-linux booting
        let pa: usize = s2tte.address(level).ok_or(Error::RmiErrorRtt(0))?.into(); //XXX: check this again
        let mut flags = 0;
        flags |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
        flags |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
        let new_s2tte = pa as u64 | flags;

        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;
    } else if s2tte.is_unassigned() || s2tte.is_assigned() {
        let pa: usize = s2tte.address(level).ok_or(Error::RmiErrorRtt(0))?.into(); //XXX: check this again
        let flags = bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
        let new_s2tte = pa as u64 | flags;

        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;
    }

    Ok(())
}

pub fn make_exclusive(id: usize, ipa: usize, level: usize) -> Result<(), Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level)); //XXX: check this again
    }

    if s2tte.is_valid(level, false) {
        // This condition is added with no-op for handling the `else` case
    } else if s2tte.is_unassigned() || s2tte.is_assigned() {
        let flags = bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::RAM);
        let new_s2tte = s2tte.get() | flags;

        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;
    } else {
        return Err(Error::RmiErrorRtt(level)); //XXX: check this again
    }

    Ok(())
}

pub fn data_create(id: usize, ipa: usize, target_pa: usize) -> Result<(), Error> {
    let level = RTT_PAGE_LEVEL;
    let (s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level));
    }

    if !s2tte.is_unassigned() {
        return Err(Error::RmiErrorRtt(RTT_PAGE_LEVEL));
    }

    let mut new_s2tte = target_pa as u64;
    if s2tte.is_invalid_ripas() {
        panic!("invalid ripas");
    }
    let ripas = s2tte.get_ripas();
    if ripas == invalid_ripas::EMPTY {
        new_s2tte |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
        new_s2tte |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
    } else if ripas == invalid_ripas::RAM {
        // S2TTE_PAGE  : S2TTE_ATTRS | S2TTE_L3_PAGE
        new_s2tte |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
        // S2TTE_ATTRS : S2TTE_MEMATTR_FWB_NORMAL_WB | S2TTE_AP_RW | S2TTE_SH_IS | S2TTE_AF
        new_s2tte |= bits_in_reg(S2TTE::MEMATTR, attribute::NORMAL_FWB);
        new_s2tte |= bits_in_reg(S2TTE::AP, permission::RW);
        new_s2tte |= bits_in_reg(S2TTE::SH, shareable::INNER);
        new_s2tte |= bits_in_reg(S2TTE::AF, 1);
    } else {
        panic!("Unexpected ripas: {}", ripas);
    }

    get_realm(id)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;

    Ok(())
}

pub fn data_destroy(id: usize, ipa: usize) -> Result<usize, Error> {
    let level = RTT_PAGE_LEVEL;
    let (s2tte, last_level) = S2TTE::get_s2tte(id, ipa, level, Error::RmiErrorRtt(0))?;

    if last_level != level {
        return Err(Error::RmiErrorRtt(last_level));
    }

    let valid = s2tte.is_valid(last_level, false);
    if !valid && !s2tte.is_assigned() {
        return Err(Error::RmiErrorRtt(RTT_PAGE_LEVEL));
    }

    let pa = s2tte
        .address(last_level)
        .ok_or(Error::RmiErrorRtt(0))?
        .into(); //XXX: check this again

    let mut flags = 0;
    if valid {
        flags |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::DESTROYED);
    } else {
        flags |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
        flags |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
    }
    let new_s2tte = flags;
    get_realm(id)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .page_table
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;

    Ok(pa)
}
