use crate::granule::GRANULE_SIZE;
use crate::granule::{set_granule, GranuleState};
use crate::mm::translation::PageTable;
use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::page_table::pte::attribute;
use crate::realm::mm::page_table::pte::{permission, shareable};
use crate::realm::mm::stage2_tte::{desc_type, invalid_hipas, invalid_ripas};
use crate::realm::mm::stage2_tte::{RttPage, INVALID_UNPROTECTED, S2TTE};
use crate::realm::rd::Rd;
use crate::rmi::error::Error;
use crate::rmi::rtt::RTT_PAGE_LEVEL;
use crate::rmi::rtt::S2TTE_STRIDE;
use crate::rmi::rtt_entry_state;
use crate::{get_granule, get_granule_if};
use armv9a::bits_in_reg;

fn level_map_size(level: usize) -> usize {
    // TODO: get the translation granule from src/armv9
    match level {
        3 => GRANULE_SIZE, // 4096
        2 => GRANULE_SIZE << S2TTE_STRIDE,
        1 => GRANULE_SIZE << (S2TTE_STRIDE * 2),
        0 => GRANULE_SIZE << (S2TTE_STRIDE * 3),
        _ => unreachable!(),
    }
}

pub fn create(rd: &Rd, rtt_addr: usize, ipa: usize, level: usize) -> Result<(), Error> {
    let mut rtt_granule = get_granule_if!(rtt_addr, GranuleState::Delegated)?;

    let s2tt = rtt_granule.content_mut::<RttPage>();

    let (parent_s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level - 1, Error::RmiErrorInput)?;

    if last_level != level - 1 {
        return Err(Error::RmiErrorRtt(last_level));
    }

    let map_size = level_map_size(level);

    let s2tt_len = s2tt.len();
    if parent_s2tte.is_unassigned() {
        if parent_s2tte.is_invalid_ripas() {
            panic!("invalid ripas");
        }
        let mut new_s2tte = bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);

        if parent_s2tte.get_masked_value(S2TTE::NS) != 0 {
            new_s2tte |= bits_in_reg(S2TTE::NS, 1);
        } else {
            let ripas = parent_s2tte.get_ripas();
            new_s2tte |= bits_in_reg(S2TTE::INVALID_RIPAS, ripas);
        }

        for i in 0..s2tt_len {
            if let Some(elem) = s2tt.get_mut(i) {
                *elem = new_s2tte;
            }
        }
    } else if parent_s2tte.is_assigned() {
        let mut flags = bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
        if parent_s2tte.is_assigned_destroyed() {
            flags |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::DESTROYED);
        } else if parent_s2tte.is_assigned_empty() {
            flags |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
        } else {
            panic!("Unexpected s2tte value:{:X}", parent_s2tte.get());
        }

        let mut pa: usize = parent_s2tte
            .address(level - 1)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again

        let mut new_s2tte = pa as u64 | flags;
        for i in 0..s2tt_len {
            if let Some(elem) = s2tt.get_mut(i) {
                *elem = new_s2tte;
            }
            pa += map_size;
            new_s2tte = pa as u64 | flags;
        }
    } else if parent_s2tte.is_assigned_ram(level - 1) {
        let mut flags = bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
        if level == RTT_PAGE_LEVEL {
            flags |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
        } else {
            flags |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_BLOCK);
        }

        let mut pa: usize = parent_s2tte
            .address(level - 1)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again

        let mut new_s2tte = pa as u64 | flags;
        for i in 0..s2tt_len {
            if let Some(elem) = s2tt.get_mut(i) {
                *elem = new_s2tte;
            }
            pa += map_size;
            new_s2tte = pa as u64 | flags;
        }
    } else if parent_s2tte.is_assigned_ns(level - 1) {
        unimplemented!();
    } else if parent_s2tte.is_table(level - 1) {
        return Err(Error::RmiErrorRtt(level - 1));
    } else {
        panic!("Unexpected s2tte value:{:X}", parent_s2tte.get());
    }

    set_granule(&mut rtt_granule, GranuleState::RTT)?;

    let parent_s2tte = rtt_addr as u64 | bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_TABLE);
    rd.s2_table()
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level - 1, parent_s2tte)?;

    // The below is added to avoid a fault regarding the RTT entry
    PageTable::get_ref().map(rtt_addr, true);

    Ok(())
}

pub fn destroy(rd: &Rd, ipa: usize, level: usize) -> Result<(usize, usize), Error> {
    let (parent_s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level - 1, Error::RmiErrorRtt(0))?;

    if (last_level != level - 1) || !parent_s2tte.is_table(level - 1) {
        return Err(Error::RmiErrorRtt(last_level));
    }

    let rtt_addr = parent_s2tte
        .address(RTT_PAGE_LEVEL)
        .ok_or(Error::RmiErrorInput)?
        .try_into()
        .or(Err(Error::RmiErrorInput))?;

    let mut g_rtt = get_granule_if!(rtt_addr, GranuleState::RTT)?;

    // TODO: granule needs to contain its refcount info.
    //       Unless its ref count is 0, RTT DESTROY should fail

    let parent_s2tte = if rd.addr_in_par(ipa) {
        bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED)
            | bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::DESTROYED)
    } else {
        bits_in_reg(S2TTE::NS, 1)
            | bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED)
            | INVALID_UNPROTECTED
    };

    rd.s2_table()
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level - 1, parent_s2tte)?;

    set_granule(&mut g_rtt, GranuleState::Delegated)?;

    let top_ipa = skip_non_live_entries(rd, ipa, level)?;
    Ok((rtt_addr, top_ipa))
}

pub fn init_ripas(rd: &Rd, base: usize, top: usize) -> Result<usize, Error> {
    // TODO: get s2tte without the level input
    let level = RTT_PAGE_LEVEL;
    let (_s2tte, last_level) = S2TTE::get_s2tte(rd, base, level, Error::RmiErrorRtt(0))?;

    let map_size = level_map_size(last_level);

    let mut addr = base & !(map_size - 1);
    if addr != base {
        warn!("base is not aligned");
        return Err(Error::RmiErrorRtt(last_level));
    }

    if top != (top & !(map_size - 1)) {
        warn!("top is not aligned");
        return Err(Error::RmiErrorRtt(last_level));
    }

    let parent_map_size = map_size << S2TTE_STRIDE;
    let top_addr = (addr & !(parent_map_size - 1)) + parent_map_size;
    while addr < top_addr {
        let next = addr + map_size;
        if next > top {
            break;
        }
        let (s2tte, last_level) = S2TTE::get_s2tte(rd, addr, level, Error::RmiErrorRtt(0))?;
        if s2tte.is_unassigned_empty() {
            let new_s2tte = bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED)
                | bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::RAM);

            rd.s2_table().lock().ipa_to_pte_set(
                GuestPhysAddr::from(addr),
                last_level,
                new_s2tte,
            )?;
        } else if !s2tte.is_unassigned_ram() {
            break;
        }
        // TODO: measurement

        addr += map_size;
    }

    if addr > base {
        Ok(addr)
    } else {
        Err(Error::RmiErrorRtt(last_level))
    }
}

pub fn get_ripas(rd: &Rd, ipa: usize, level: usize) -> Result<u64, Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level));
    }

    if s2tte.is_destroyed() {
        error!("The s2tte is destroyed: {:x}", s2tte.get());
        return Err(Error::RmiErrorRtt(last_level));
    }
    Ok(s2tte.get_ripas())
}

pub fn read_entry(rd: &Rd, ipa: usize, level: usize) -> Result<[usize; 4], Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

    let r1 = last_level;
    let (mut r2, mut r3, mut r4) = (0, 0, 0);

    if s2tte.is_unassigned() {
        r2 = rtt_entry_state::RMI_UNASSIGNED;
        r4 = if s2tte.is_unassigned_ns() {
            invalid_ripas::EMPTY as usize
        } else {
            s2tte.get_masked_value(S2TTE::INVALID_RIPAS) as usize
        };
    } else if s2tte.is_assigned_empty() {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        r3 = s2tte
            .address(last_level)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
        r4 = invalid_ripas::EMPTY as usize;
    } else if s2tte.is_assigned_destroyed() {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        r4 = invalid_ripas::DESTROYED as usize;
    } else if s2tte.is_assigned_ram(last_level) {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        r3 = s2tte
            .address(last_level)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
        r4 = invalid_ripas::RAM as usize;
    } else if s2tte.is_assigned_ns(last_level) {
        r2 = rtt_entry_state::RMI_ASSIGNED;
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
        r4 = invalid_ripas::EMPTY as usize;
    } else if s2tte.is_table(last_level) {
        r2 = rtt_entry_state::RMI_TABLE;
        r3 = s2tte
            .address(RTT_PAGE_LEVEL)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
        r4 = invalid_ripas::EMPTY as usize;
    } else {
        error!("Unexpected S2TTE value retrieved!");
    }
    Ok([r1, r2, r3, r4])
}

pub fn map_unprotected(rd: &Rd, ipa: usize, level: usize, host_s2tte: usize) -> Result<(), Error> {
    if rd.addr_in_par(ipa) {
        return Err(Error::RmiErrorInput);
    }

    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

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

    rd.s2_table()
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;

    Ok(())
}

pub fn unmap_unprotected(rd: &Rd, ipa: usize, level: usize) -> Result<usize, Error> {
    if rd.addr_in_par(ipa) {
        return Err(Error::RmiErrorInput);
    }

    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level));
    }

    if !s2tte.is_assigned_ns(level) {
        return Err(Error::RmiErrorRtt(level));
    }

    let new_s2tte: u64 = bits_in_reg(S2TTE::NS, 1)
        | bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED)
        | INVALID_UNPROTECTED;

    rd.s2_table()
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;

    //TODO: add page/block invalidation
    /*
    if level == RTT_PAGE_LEVEL {
        // invalidate the page
    } else {
        // invalidate the block
    }
    */

    let top_ipa = skip_non_live_entries(rd, ipa, level)?;
    Ok(top_ipa)
}

pub fn make_shared(rd: &Rd, ipa: usize, level: usize) -> Result<(), Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

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

        rd.s2_table()
            .lock()
            .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;
    } else if s2tte.is_unassigned() || s2tte.is_assigned() {
        let pa: usize = s2tte.address(level).ok_or(Error::RmiErrorRtt(0))?.into(); //XXX: check this again
        let flags = bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
        let new_s2tte = pa as u64 | flags;

        rd.s2_table()
            .lock()
            .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;
    }

    Ok(())
}

pub fn make_exclusive(rd: &Rd, ipa: usize, level: usize) -> Result<(), Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level {
        return Err(Error::RmiErrorRtt(last_level)); //XXX: check this again
    }

    if s2tte.is_valid(level, false) {
        // This condition is added with no-op for handling the `else` case
    } else if s2tte.is_unassigned() || s2tte.is_assigned() {
        let flags = bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::RAM);
        let new_s2tte = s2tte.get() | flags;

        rd.s2_table()
            .lock()
            .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;
    } else {
        return Err(Error::RmiErrorRtt(level)); //XXX: check this again
    }

    Ok(())
}

pub fn data_create(rd: &Rd, ipa: usize, target_pa: usize, unknown: bool) -> Result<(), Error> {
    let level = RTT_PAGE_LEVEL;
    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

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
    if unknown && ripas != invalid_ripas::RAM {
        new_s2tte |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
        new_s2tte |= bits_in_reg(S2TTE::INVALID_RIPAS, ripas);
    } else {
        // S2TTE_PAGE  : S2TTE_ATTRS | S2TTE_L3_PAGE
        new_s2tte |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
        // S2TTE_ATTRS : S2TTE_MEMATTR_FWB_NORMAL_WB | S2TTE_AP_RW | S2TTE_SH_IS | S2TTE_AF
        new_s2tte |= bits_in_reg(S2TTE::MEMATTR, attribute::NORMAL_FWB);
        new_s2tte |= bits_in_reg(S2TTE::AP, permission::RW);
        new_s2tte |= bits_in_reg(S2TTE::SH, shareable::INNER);
        new_s2tte |= bits_in_reg(S2TTE::AF, 1);
    }

    rd.s2_table()
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;

    Ok(())
}

pub fn data_destroy(rd: &Rd, ipa: usize) -> Result<(usize, usize), Error> {
    let level = RTT_PAGE_LEVEL;
    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

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
    if s2tte.is_assigned_ram(RTT_PAGE_LEVEL) {
        flags |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
        flags |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::DESTROYED);
        // TODO: call invalidate page
    } else if s2tte.is_assigned_empty() {
        flags |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
        flags |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
    } else if s2tte.is_assigned_destroyed() {
        flags |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
        flags |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::DESTROYED);
    }
    let new_s2tte = flags;
    rd.s2_table()
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte)?;

    let top_ipa = skip_non_live_entries(rd, ipa, level)?;

    Ok((pa, top_ipa))
}

fn skip_non_live_entries(rd: &Rd, base: usize, level: usize) -> Result<usize, Error> {
    let map_size = level_map_size(level);

    let mut addr = base & !(map_size - 1);
    if addr != base {
        return Err(Error::RmiErrorRtt(level));
    }

    let parent_map_size = map_size << S2TTE_STRIDE;
    let top_addr = (addr & !(parent_map_size - 1)) + parent_map_size;
    while addr < top_addr {
        let (s2tte, _last_level) = S2TTE::get_s2tte(rd, addr, level, Error::RmiErrorRtt(level))?;
        if s2tte.is_live(level) {
            break;
        }
        addr += map_size;
    }
    Ok(addr)
}
