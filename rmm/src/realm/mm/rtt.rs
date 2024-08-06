use crate::granule::GRANULE_SHIFT;
use crate::granule::{set_granule, GranuleState};
use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::attribute::{desc_type, memattr, permission, shareable};
use crate::realm::mm::entry;
use crate::realm::mm::stage2_translation::{RttAllocator, Tlbi};
use crate::realm::mm::stage2_tte::{invalid_hipas, invalid_ripas, mapping_size, S2TTE};
use crate::realm::mm::stage2_tte::{level_mask, INVALID_UNPROTECTED};
use crate::realm::mm::table_level;
use crate::realm::rd::Rd;
use crate::rmi::error::Error;
use crate::rmi::rtt_entry_state;
use crate::{get_granule, get_granule_if};
use armv9a::bits_in_reg;
use vmsa::address::PhysAddr;
use vmsa::page_table::{Entry, Level, PageTable};

pub const RTT_MIN_BLOCK_LEVEL: usize = table_level::L2Table::THIS_LEVEL;
pub const RTT_PAGE_LEVEL: usize = table_level::L3Table::THIS_LEVEL;
pub const RTT_STRIDE: usize = GRANULE_SHIFT - 3;

const CHANGE_DESTROYED: u64 = 0x1;

fn level_space_size(rd: &Rd, level: usize) -> usize {
    rd.s2_table().lock().space_size(level)
}

fn create_pgtbl_at(
    rtt_addr: usize,
    flags: u64,
    mut pa: usize,
    map_size: usize,
) -> Result<(), Error> {
    let alloc = RttAllocator { base: rtt_addr };
    let mut new_s2tte = pa as u64 | flags;

    let ret = PageTable::<
        GuestPhysAddr,
        table_level::L3Table, //Table Level is not meaninful here
        entry::Entry,
        { table_level::L3Table::NUM_ENTRIES },
    >::new_init_in(&alloc, |entries| {
        for e in entries.iter_mut() {
            let _ = (*e).set(PhysAddr::from(pa), new_s2tte);
        }
        pa += map_size;
        new_s2tte = pa as u64 | flags;
    });

    if ret.is_err() {
        return Err(Error::RmiErrorRtt(0));
    }
    Ok(())
}

pub fn create(rd: &Rd, rtt_addr: usize, ipa: usize, level: usize) -> Result<(), Error> {
    let mut invalidate = Tlbi::NONE;

    let (parent_s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level - 1, Error::RmiErrorInput)?;

    if last_level != level - 1 {
        return Err(Error::RmiErrorRtt(last_level));
    }

    let map_size = mapping_size(level);

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

        create_pgtbl_at(rtt_addr, new_s2tte, 0, 0)?;
    } else if parent_s2tte.is_assigned() {
        if parent_s2tte.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::RAM {
            panic!("Unexpected s2tte value:{:X}", parent_s2tte.get());
        }

        let flags = parent_s2tte.get_masked(S2TTE::INVALID_HIPAS | S2TTE::INVALID_RIPAS);

        let pa: usize = parent_s2tte
            .addr_as_block(level - 1)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again

        create_pgtbl_at(rtt_addr, flags, pa, map_size)?;
    } else if parent_s2tte.is_assigned_ram(level - 1) {
        let mut flags = bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
        if level == RTT_PAGE_LEVEL {
            flags |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
        } else {
            flags |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_BLOCK);
        }

        let pa: usize = parent_s2tte
            .addr_as_block(level - 1)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again

        create_pgtbl_at(rtt_addr, flags, pa, map_size)?;
        invalidate = Tlbi::LEAF(rd.id());
    } else if parent_s2tte.is_assigned_ns(level - 1) {
        unimplemented!();
    } else if parent_s2tte.is_table(level - 1) {
        return Err(Error::RmiErrorRtt(level - 1));
    } else {
        panic!("Unexpected s2tte value:{:X}", parent_s2tte.get());
    }

    let parent_s2tte = rtt_addr as u64 | bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_TABLE);
    rd.s2_table().lock().ipa_to_pte_set(
        GuestPhysAddr::from(ipa),
        level - 1,
        parent_s2tte,
        invalidate,
    )?;

    // The below is added to avoid a fault regarding the RTT entry
    crate::mm::translation::PageTable::get_ref().map(rtt_addr, true);

    Ok(())
}

pub fn destroy<F: FnMut(usize)>(
    rd: &Rd,
    ipa: usize,
    level: usize,
    mut f: F,
) -> Result<(usize, usize), Error> {
    let invalidate;
    let (parent_s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level - 1, Error::RmiErrorRtt(0))?;

    if (last_level != level - 1) || !parent_s2tte.is_table(last_level) {
        let top_ipa = skip_non_live_entries(rd, ipa, last_level)?;
        f(top_ipa);
        return Err(Error::RmiErrorRtt(last_level));
    }

    let rtt_addr = parent_s2tte
        .addr_as_block(RTT_PAGE_LEVEL)
        .ok_or(Error::RmiErrorInput)?
        .into();

    let mut g_rtt = get_granule_if!(rtt_addr, GranuleState::RTT)?;

    // TODO: granule needs to contain its refcount info.
    //       Unless its ref count is 0, RTT DESTROY should fail

    let parent_s2tte = if rd.addr_in_par(ipa) {
        invalidate = Tlbi::LEAF(rd.id());
        bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED)
            | bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::DESTROYED)
    } else {
        invalidate = Tlbi::BREAKDOWN(rd.id());
        bits_in_reg(S2TTE::NS, 1)
            | bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED)
            | INVALID_UNPROTECTED
    };

    rd.s2_table().lock().ipa_to_pte_set(
        GuestPhysAddr::from(ipa),
        level - 1,
        parent_s2tte,
        invalidate,
    )?;

    set_granule(&mut g_rtt, GranuleState::Delegated)?;

    let top_ipa = skip_non_live_entries(rd, ipa, last_level)?;
    Ok((rtt_addr, top_ipa))
}

pub fn init_ripas(rd: &Rd, base: usize, top: usize) -> Result<usize, Error> {
    // TODO: get s2tte without the level input
    let level = RTT_PAGE_LEVEL;
    let (_s2tte, last_level) = S2TTE::get_s2tte(rd, base, level, Error::RmiErrorRtt(0))?;

    let map_size = mapping_size(last_level);

    let mut addr = base & !(map_size - 1);
    if addr != base {
        warn!("base is not aligned");
        return Err(Error::RmiErrorRtt(last_level));
    }

    if top != (top & !(map_size - 1)) {
        warn!("top is not aligned");
        return Err(Error::RmiErrorRtt(last_level));
    }

    let space_size = level_space_size(rd, last_level);
    let top_addr = (addr & !(space_size - 1)) + space_size;
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
                Tlbi::NONE,
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

    if s2tte.is_assigned_ram(level) {
        return Ok(invalid_ripas::RAM);
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
            .addr_as_block(last_level)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
        r4 = invalid_ripas::EMPTY as usize;
    } else if s2tte.is_assigned_destroyed() {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        r4 = invalid_ripas::DESTROYED as usize;
    } else if s2tte.is_assigned_ram(last_level) {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        r3 = s2tte
            .addr_as_block(last_level)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
        r4 = invalid_ripas::RAM as usize;
    } else if s2tte.is_assigned_ns(last_level) {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        let addr_mask: u64 = level_mask(level).ok_or(Error::RmiErrorRtt(0))?;
        let mask = addr_mask | S2TTE::MEMATTR | S2TTE::S2AP | S2TTE::SH;
        r3 = s2tte.get_masked(mask) as usize;
        r4 = invalid_ripas::EMPTY as usize;
    } else if s2tte.is_table(last_level) {
        r2 = rtt_entry_state::RMI_TABLE;
        r3 = s2tte
            .addr_as_block(RTT_PAGE_LEVEL)
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

    // TODO: should return actual last level, not level 0
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

    rd.s2_table().lock().ipa_to_pte_set(
        GuestPhysAddr::from(ipa),
        level,
        new_s2tte,
        Tlbi::LEAF(rd.id()),
    )?;

    Ok(())
}

pub fn unmap_unprotected<F: FnMut(usize)>(
    rd: &Rd,
    ipa: usize,
    level: usize,
    mut f: F,
) -> Result<usize, Error> {
    if rd.addr_in_par(ipa) {
        return Err(Error::RmiErrorInput);
    }

    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

    if level != last_level || !s2tte.is_assigned_ns(last_level) {
        let top_ipa = skip_non_live_entries(rd, ipa, last_level)?;
        f(top_ipa);
        return Err(Error::RmiErrorRtt(last_level));
    }

    let new_s2tte: u64 = bits_in_reg(S2TTE::NS, 1)
        | bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED)
        | INVALID_UNPROTECTED;

    rd.s2_table().lock().ipa_to_pte_set(
        GuestPhysAddr::from(ipa),
        level,
        new_s2tte,
        Tlbi::LEAF(rd.id()),
    )?;

    let top_ipa = skip_non_live_entries(rd, ipa, level)?;
    Ok(top_ipa)
}

pub fn set_ripas(rd: &Rd, base: usize, top: usize, ripas: u8, flags: u64) -> Result<usize, Error> {
    // TODO: get it from s2table with the start address
    let level = RTT_PAGE_LEVEL;
    let (_s2tte, level) = S2TTE::get_s2tte(rd, base, RTT_PAGE_LEVEL, Error::RmiErrorRtt(level))?;

    let map_size = mapping_size(level);

    let mut addr = base & !(map_size - 1);
    if addr != base {
        return Err(Error::RmiErrorRtt(level));
    }
    if top & !(map_size - 1) != top {
        return Err(Error::RmiErrorRtt(level));
    }
    let space_size = level_space_size(rd, level);
    let table_top = (addr & !(space_size - 1)) + space_size;
    if table_top < top {
        debug!(
            "table can address upto 0x{:X}, top {:X} overlimits the range",
            table_top, top
        );
    }

    while addr < table_top && addr < top {
        let mut invalidate = Tlbi::NONE;
        let (s2tte, last_level) =
            S2TTE::get_s2tte(rd, addr, RTT_PAGE_LEVEL, Error::RmiErrorRtt(level))?;
        let mut new_s2tte = 0;
        let mut add_pa = false;

        if level != last_level {
            break;
        }
        let pa: usize = s2tte
            .addr_as_block(last_level)
            .ok_or(Error::RmiErrorRtt(0))?
            .into(); //XXX: check this again
        if ripas as u64 == invalid_ripas::EMPTY {
            new_s2tte |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);

            if s2tte.is_unassigned_empty() {
                new_s2tte |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
            } else if s2tte.is_unassigned_destroyed() {
                if flags & CHANGE_DESTROYED != 0 {
                    new_s2tte |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
                } else {
                    break;
                }
            } else if s2tte.is_assigned_ram(level) {
                new_s2tte |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
                add_pa = true;
                invalidate = Tlbi::LEAF(rd.id());
            } else if s2tte.is_assigned_destroyed() {
                if flags & CHANGE_DESTROYED != 0 {
                    new_s2tte |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
                    add_pa = true;
                    invalidate = Tlbi::LEAF(rd.id());
                } else {
                    break;
                }
            }
        } else if ripas as u64 == invalid_ripas::RAM {
            new_s2tte |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::RAM);

            if s2tte.is_unassigned_empty() {
                new_s2tte |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
            } else if s2tte.is_unassigned_destroyed() {
                if flags & CHANGE_DESTROYED != 0 {
                    new_s2tte |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
                } else {
                    break;
                }
            } else if s2tte.is_assigned_empty() {
                //assigned ram
                new_s2tte |= bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
                if last_level == RTT_PAGE_LEVEL {
                    new_s2tte |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
                } else {
                    new_s2tte |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_BLOCK);
                }
                add_pa = true;
            } else if s2tte.is_assigned_destroyed() {
                if flags & CHANGE_DESTROYED != 0 {
                    //assigned ram
                    if last_level == RTT_PAGE_LEVEL {
                        new_s2tte |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
                    } else {
                        new_s2tte |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_BLOCK);
                    }
                    add_pa = true;
                } else {
                    break;
                }
            } else {
                addr += map_size;
                continue; // do nothing
            }
        } else {
            unreachable!();
        }
        if add_pa {
            new_s2tte |= pa as u64;
        }
        rd.s2_table().lock().ipa_to_pte_set(
            GuestPhysAddr::from(addr),
            last_level,
            new_s2tte,
            invalidate,
        )?;

        addr += map_size;
    }
    if addr > base {
        Ok(addr)
    } else {
        Err(Error::RmiErrorRtt(level))
    }
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
        new_s2tte |= bits_in_reg(S2TTE::MEMATTR, memattr::NORMAL_FWB);
        new_s2tte |= bits_in_reg(S2TTE::S2AP, permission::RW);
        new_s2tte |= bits_in_reg(S2TTE::SH, shareable::INNER);
        new_s2tte |= bits_in_reg(S2TTE::AF, 1);
    }

    rd.s2_table()
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte, Tlbi::NONE)?;

    Ok(())
}

pub fn data_destroy<F: FnMut(usize)>(
    rd: &Rd,
    ipa: usize,
    mut f: F,
) -> Result<(usize, usize), Error> {
    let mut invalidate = Tlbi::NONE;
    let level = RTT_PAGE_LEVEL;
    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(level))?;

    let valid = s2tte.is_valid(last_level, false);
    if last_level < level || (!valid && !s2tte.is_assigned()) {
        let top_ipa = skip_non_live_entries(rd, ipa, last_level)?;
        f(top_ipa);
        return Err(Error::RmiErrorRtt(last_level));
    }

    let pa = s2tte
        .addr_as_block(last_level)
        .ok_or(Error::RmiErrorRtt(last_level))?
        .into(); //XXX: check this again

    let mut flags = bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
    if s2tte.is_assigned_ram(RTT_PAGE_LEVEL) {
        flags |= bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::DESTROYED);
        invalidate = Tlbi::LEAF(rd.id());
    } else {
        flags |= s2tte.get_masked(S2TTE::INVALID_RIPAS);
    }
    let new_s2tte = flags;
    rd.s2_table()
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte, invalidate)?;

    let top_ipa = skip_non_live_entries(rd, ipa, level)?;

    Ok((pa, top_ipa))
}

fn skip_non_live_entries(rd: &Rd, base: usize, level: usize) -> Result<usize, Error> {
    let map_size = mapping_size(level);

    let mut addr = base & !(map_size - 1);
    if addr != base {
        return Err(Error::RmiErrorRtt(level));
    }

    let space_size = level_space_size(rd, level);
    let mut bottom_addr = addr & !(space_size - 1);

    let binding = rd.s2_table();
    let binding = binding.lock();
    let (entries_iter, last_level) = binding.entries(GuestPhysAddr::from(base), level)?;
    if level != last_level {
        warn!(
            "level doesn't match! level:{:?} last_level:{:?}",
            level, last_level
        );
    }
    for entry in entries_iter {
        if bottom_addr < base {
            bottom_addr += map_size;
            continue;
        }
        let s2tte = S2TTE::new(entry.pte());
        if s2tte.is_live(level) {
            return Ok(addr);
        }
        addr += map_size;
    }
    Ok(addr)
}
