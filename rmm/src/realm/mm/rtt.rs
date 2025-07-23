use crate::granule::GRANULE_SHIFT;
use crate::granule::{set_granule, GranuleState};
use crate::measurement::HashContext;
use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::attribute::desc_type;
use crate::realm::mm::entry;
use crate::realm::mm::stage2_translation::{RttAllocator, Tlbi};
use crate::realm::mm::stage2_tte::{hipas, mapping_size, ripas, S2TTE};
use crate::realm::mm::stage2_tte::{
    level_mask, INVALID_UNPROTECTED, TABLE_TTE, VALID_NS_TTE, VALID_TTE,
};
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
            pa += map_size;
            new_s2tte = pa as u64 | flags;
        }
    });

    if ret.is_err() {
        return Err(Error::RmiErrorRtt(0));
    }
    Ok(())
}

pub fn create(rd: &Rd, rtt_addr: usize, ipa: usize, level: usize) -> Result<(), Error> {
    let mut invalidate = Tlbi::NONE;

    let (parent_s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level - 1, Error::RmiErrorInput)?;

    if last_level != level - 1 || parent_s2tte.is_table(last_level) {
        return Err(Error::RmiErrorRtt(last_level));
    }

    let map_size = mapping_size(level);
    let mut flags = parent_s2tte.get_masked(S2TTE::HIPAS);
    if rd.addr_in_par(ipa) {
        flags |= parent_s2tte.get_masked(S2TTE::RIPAS);
    } else {
        flags |= parent_s2tte.get_masked(S2TTE::NS);
    }

    if parent_s2tte.is_unassigned() || parent_s2tte.is_unassigned_ns() {
        create_pgtbl_at(rtt_addr, flags, 0, 0)?;
    } else {
        if level == RTT_PAGE_LEVEL {
            flags |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
        } else {
            flags |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_BLOCK);
        }
        let pa: usize = parent_s2tte.addr_as_block(level - 1).into(); //XXX: check this again
        flags |= parent_s2tte.get_masked(S2TTE::MEMATTR | S2TTE::S2AP | S2TTE::SH);
        if parent_s2tte.is_assigned_ram(level - 1) {
            invalidate = Tlbi::LEAF(rd.id());
        } else if !parent_s2tte.is_assigned_ns(level - 1) {
            panic!("Unexpected s2tte value:{:X}", parent_s2tte.get());
        }
        create_pgtbl_at(rtt_addr, flags, pa, map_size)?;
    }

    let parent_s2tte = rtt_addr as u64 | TABLE_TTE;
    rd.s2_table().lock().ipa_to_pte_set(
        GuestPhysAddr::from(ipa),
        level - 1,
        parent_s2tte,
        invalidate,
    )?;

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

    let rtt_addr = parent_s2tte.addr_as_block(RTT_PAGE_LEVEL).into();

    let mut g_rtt = get_granule_if!(rtt_addr, GranuleState::RTT)?;

    // TODO: granule needs to contain its refcount info.
    //       Unless its ref count is 0, RTT DESTROY should fail
    if is_live_rtt(rd, ipa, level).unwrap_or(false) {
        f(ipa);
        return Err(Error::RmiErrorRtt(level));
    }

    let parent_s2tte = if rd.addr_in_par(ipa) {
        invalidate = Tlbi::LEAF(rd.id());
        bits_in_reg(S2TTE::HIPAS, hipas::UNASSIGNED) | bits_in_reg(S2TTE::RIPAS, ripas::DESTROYED)
    } else {
        invalidate = Tlbi::BREAKDOWN(rd.id());
        bits_in_reg(S2TTE::NS, 1)
            | bits_in_reg(S2TTE::HIPAS, hipas::UNASSIGNED)
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

pub fn init_ripas(rd: &mut Rd, base: usize, top: usize) -> Result<usize, Error> {
    // TODO: get s2tte without the level input
    let level = RTT_PAGE_LEVEL;
    let (s2tte, last_level) = S2TTE::get_s2tte(rd, base, level, Error::RmiErrorRtt(0))?;

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

    if s2tte.get_masked_value(S2TTE::HIPAS) != hipas::UNASSIGNED {
        warn!("base is assigned already");
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
        if s2tte.is_table(last_level) || s2tte.get_masked_value(S2TTE::HIPAS) == hipas::ASSIGNED {
            break;
        }
        let new_s2tte =
            bits_in_reg(S2TTE::HIPAS, hipas::UNASSIGNED) | bits_in_reg(S2TTE::RIPAS, ripas::RAM);

        rd.s2_table().lock().ipa_to_pte_set(
            GuestPhysAddr::from(addr),
            last_level,
            new_s2tte,
            Tlbi::NONE,
        )?;

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(rd)?.measure_ripas_granule(addr as u64, next as u64)?;

        addr += map_size;
    }

    if addr > base {
        Ok(addr)
    } else {
        Err(Error::RmiErrorRtt(last_level))
    }
}

// return (out_top, ripas)
pub fn get_ripas(rd: &Rd, start: usize, end: usize) -> Result<(usize, u64), Error> {
    let level = RTT_PAGE_LEVEL;
    let mut addr = start;
    let mut common_ripas = 0; // initialized in the below if condition (addr == start)
    let mut map_size = 0; // initialized in the below if condition (addr == start)
    while addr < end {
        let (s2tte, last_level) = S2TTE::get_s2tte(rd, addr, level, Error::RmiErrorRtt(0))?;
        if !s2tte.has_ripas(level) {
            break;
        }
        let ripas = s2tte.get_ripas();
        if addr == start {
            common_ripas = ripas;
            map_size = mapping_size(last_level);
        } else if common_ripas != ripas {
            break;
        }
        addr += map_size;
    }
    if addr == start {
        return Err(Error::RmiErrorInput);
    }
    Ok((addr, common_ripas))
}

pub fn read_entry(rd: &Rd, ipa: usize, level: usize) -> Result<[usize; 4], Error> {
    let (s2tte, last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(0))?;

    let r1 = last_level;
    let (mut r2, mut r3, mut r4) = (0, 0, 0);

    if s2tte.is_unassigned() {
        r2 = rtt_entry_state::RMI_UNASSIGNED;
        r4 = s2tte.get_masked_value(S2TTE::RIPAS) as usize;
    } else if s2tte.is_assigned() || s2tte.is_assigned_ram(last_level) {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        r3 = s2tte.addr_as_block(last_level).into(); //XXX: check this again
        r4 = s2tte.get_masked_value(S2TTE::RIPAS) as usize;
    } else if s2tte.is_table(last_level) {
        r2 = rtt_entry_state::RMI_TABLE;
        r3 = s2tte.get_masked(S2TTE::ADDR_TBL_OR_PAGE); //XXX: check this again
    } else if s2tte.is_unassigned_ns() {
        r2 = rtt_entry_state::RMI_UNASSIGNED;
    } else if s2tte.is_assigned_ns(last_level) {
        r2 = rtt_entry_state::RMI_ASSIGNED;
        let addr_mask: u64 = level_mask(last_level).ok_or(Error::RmiErrorRtt(0))?;
        let mask = addr_mask | S2TTE::MEMATTR | S2TTE::S2AP | S2TTE::SH;
        r3 = s2tte.get_masked(mask);
    } else {
        error!("Unexpected S2TTE value retrieved!");
    }
    Ok([r1, r2, r3 as usize, r4])
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

    if !s2tte.is_unassigned_ns() {
        return Err(Error::RmiErrorRtt(level));
    }

    let mut new_s2tte = host_s2tte as u64 | bits_in_reg(S2TTE::HIPAS, hipas::ASSIGNED);
    if level == RTT_PAGE_LEVEL {
        new_s2tte |= VALID_NS_TTE | bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
    } else {
        new_s2tte |= VALID_NS_TTE | bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_BLOCK);
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
        | bits_in_reg(S2TTE::HIPAS, hipas::UNASSIGNED)
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
    if addr != base || top & !(map_size - 1) != top {
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
        let (s2tte, last_level) = S2TTE::get_s2tte(rd, addr, level, Error::RmiErrorRtt(level))?;
        let mut new_s2tte = 0;
        let mut add_pa = false;

        if s2tte.is_table(last_level) || s2tte.is_destroyed() && flags & CHANGE_DESTROYED == 0 {
            break;
        }
        let pa: usize = s2tte.addr_as_block(last_level).into(); //XXX: check this again
        new_s2tte |= s2tte.get_masked(S2TTE::HIPAS);
        new_s2tte |= bits_in_reg(S2TTE::RIPAS, ripas as u64);
        // If requested riaps  == current ripas, skip it.
        if ripas == s2tte.get_masked_value(S2TTE::RIPAS) as u8 {
            addr += map_size;
            continue;
        }
        if ripas as u64 == ripas::EMPTY {
            if s2tte.is_assigned_ram(last_level) {
                add_pa = true;
                invalidate = Tlbi::LEAF(rd.id());
            } else if s2tte.is_assigned_destroyed() {
                add_pa = true;
            }
        } else if ripas as u64 == ripas::RAM {
            if s2tte.is_assigned() {
                new_s2tte |= VALID_TTE;
                if last_level == RTT_PAGE_LEVEL {
                    new_s2tte |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
                } else {
                    new_s2tte |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L012_BLOCK);
                }
                add_pa = true;
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
    if s2tte.is_ripas() {
        panic!("invalid ripas");
    }
    let ripas = s2tte.get_ripas();
    // New HIPAS: ASSIGNED
    new_s2tte |= bits_in_reg(S2TTE::HIPAS, hipas::ASSIGNED);
    if unknown && ripas != ripas::RAM {
        // New RIPAS: Unchanged
        new_s2tte |= bits_in_reg(S2TTE::RIPAS, ripas);
    } else {
        // New RIPAS: RAM
        new_s2tte |= bits_in_reg(S2TTE::RIPAS, ripas::RAM);
        // S2TTE_PAGE  : S2TTE_ATTRS | S2TTE_L3_PAGE
        new_s2tte |= bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
        // S2TTE_ATTRS : S2TTE_MEMATTR_FWB_NORMAL_WB | S2TTE_AP_RW | S2TTE_SH_IS | S2TTE_AF
        new_s2tte |= VALID_TTE;
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

    let pa = s2tte.addr_as_block(last_level).into(); //XXX: check this again

    let mut new_s2tte = bits_in_reg(S2TTE::HIPAS, hipas::UNASSIGNED);
    if s2tte.get_masked(S2TTE::RIPAS) == ripas::EMPTY {
        new_s2tte |= bits_in_reg(S2TTE::RIPAS, ripas::EMPTY);
    } else {
        new_s2tte |= bits_in_reg(S2TTE::RIPAS, ripas::DESTROYED);
    }
    if s2tte.is_assigned_ram(RTT_PAGE_LEVEL) {
        invalidate = Tlbi::LEAF(rd.id());
    }
    rd.s2_table()
        .lock()
        .ipa_to_pte_set(GuestPhysAddr::from(ipa), level, new_s2tte, invalidate)?;

    let top_ipa = skip_non_live_entries(rd, ipa, level)?;

    Ok((pa, top_ipa))
}

fn is_live_rtt(rd: &Rd, base: usize, level: usize) -> Result<bool, Error> {
    let binding = rd.s2_table();
    let binding = binding.lock();
    let (entries_iter, last_level) = binding.entries(GuestPhysAddr::from(base), level)?;
    if level != last_level {
        error!(
            "level doesn't match! level:{:?} last_level:{:?}",
            level, last_level
        );
    }

    for entry in entries_iter {
        let s2tte = S2TTE::new(entry.pte());
        if s2tte.is_live(level) {
            return Ok(true);
        }
    }
    Err(Error::RmiErrorRtt(level))
}

fn skip_non_live_entries(rd: &Rd, base: usize, level: usize) -> Result<usize, Error> {
    let map_size = mapping_size(level);

    let mut addr = base & !(map_size - 1);
    let space_size = level_space_size(rd, level);
    let mut entry0_ipa = addr & !(space_size - 1);

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
        // skip entries less than the base ipa
        if entry0_ipa < base {
            entry0_ipa += map_size;
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

pub fn fold(rd: &Rd, ipa: usize, level: usize) -> Result<usize, Error> {
    let is_protected_ipa = rd.addr_in_par(ipa);
    let (fold_s2tte, _last_level) = S2TTE::get_s2tte(rd, ipa, level, Error::RmiErrorRtt(level))?;
    let (parent_s2tte, parent_level) = S2TTE::get_s2tte(rd, ipa, level - 1, Error::RmiErrorInput)?;
    if parent_level < (level - 1) || !parent_s2tte.is_table(level - 1) {
        return Err(Error::RmiErrorRtt(parent_level));
    }
    // TODO: spec doesn't reject the fold with its state in TABLE.
    if fold_s2tte.is_table(level) {
        warn!("Trying to fold which points another RTT");
        return Err(Error::RmiErrorRtt(level));
    }

    let rtt_addr = parent_s2tte.get_masked(S2TTE::ADDR_TBL_OR_PAGE);
    let mut g_rtt = get_granule_if!(rtt_addr as usize, GranuleState::RTT)?;

    // TODO: ref count check

    let binding = rd.s2_table();
    let mut binding = binding.lock();
    let (mut entries_iter, _) = binding.entries(GuestPhysAddr::from(ipa), level)?;
    if !S2TTE::is_homogeneous(&mut entries_iter, level) {
        return Err(Error::RmiErrorRtt(level));
    }
    let mut pa: u64 = 0;
    let mut attr = fold_s2tte.get_masked(S2TTE::NS);
    let hipas = fold_s2tte.get_masked(S2TTE::HIPAS);
    let mut ripas = 0;
    let mut desc_type = 0;

    if fold_s2tte.get_masked_value(S2TTE::HIPAS) == hipas::ASSIGNED {
        pa = fold_s2tte.addr_as_block(level).into();
        attr |= fold_s2tte.get_masked(S2TTE::MEMATTR | S2TTE::S2AP | S2TTE::SH);
    }
    if is_protected_ipa {
        ripas = fold_s2tte.get_masked(S2TTE::RIPAS);
    }
    if fold_s2tte.is_assigned_ram(level) || fold_s2tte.is_assigned_ns(level) {
        desc_type = desc_type::L012_BLOCK;
    }

    let parent_s2tte = pa | attr | hipas | ripas | desc_type;
    binding.ipa_to_pte_set(
        GuestPhysAddr::from(ipa),
        level - 1,
        parent_s2tte,
        Tlbi::BREAKDOWN(rd.id()),
    )?;
    //Change state of child table (pa)
    set_granule(&mut g_rtt, GranuleState::Delegated)?;
    Ok(rtt_addr as usize)
}
