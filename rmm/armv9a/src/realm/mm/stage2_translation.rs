use super::page::BasePageSize;
use super::page_table::{entry, L1Table};

use core::arch::asm;
use core::ffi::c_void;
use core::fmt;

use monitor::mm::address::PhysAddr;
use monitor::mm::page::{Page, PageIter, PageSize};
use monitor::mm::page_table::Entry;
use monitor::mm::page_table::{Level, PageTable, PageTableMethods};
use monitor::realm::mm::address::GuestPhysAddr;
use monitor::realm::mm::IPATranslation;
use monitor::rmi::error::Error;

use crate::helper;
use crate::helper::bits_in_reg;
use crate::realm::mm::page_table::pte;
use crate::realm::mm::translation_granule_4k::RawPTE;
use crate::{define_bitfield, define_bits, define_mask};

// initial lookup starts at level 1 with 2 page tables concatenated
pub const NUM_ROOT_PAGE: usize = 2;
pub const ALIGN_ROOT_PAGE: usize = 2;

pub mod tlbi_ns {
    pub const IPAS_S: u64 = 0b0;
    pub const IPAS_NS: u64 = 0b1;
}

define_bits!(TLBI_OP, NS[63 - 63], TTL[47 - 44], IPA[35 - 0]);

pub struct Stage2Translation<'a> {
    // We will set the translation granule with 4KB.
    // To reduce the level of page lookup, initial lookup will start from L1.
    // We allocate two single page table initial lookup table, addresing up 1TB.
    root_pgtlb: &'a mut PageTable<
        GuestPhysAddr,
        L1Table,
        entry::Entry,
        { <L1Table as Level>::NUM_ENTRIES },
    >,
    dirty: bool,
}

impl<'a> Stage2Translation<'a> {
    pub fn new() -> Self {
        let root_pgtlb = unsafe {
            &mut *PageTable::<
                GuestPhysAddr,
                L1Table,
                entry::Entry,
                { <L1Table as Level>::NUM_ENTRIES },
            >::new_with_align(NUM_ROOT_PAGE, ALIGN_ROOT_PAGE)
            .unwrap()
        };
        fill_stage2_table(root_pgtlb);

        Self {
            root_pgtlb,
            dirty: false,
        }
    }

    // According to DDI0608A E1.2.1.11 Cache and TLB operations
    // 'TLBI IPAS2E1, Xt; DSB; TLBI VMALLE1'
    // or TLBI ALL or TLBI VMALLS1S2
    fn tlb_flush_by_vmid_ipa<S: PageSize>(&mut self, guest_iter: PageIter<S, GuestPhysAddr>) {
        for guest in guest_iter {
            let _level: u64 = S::MAP_TABLE_LEVEL as u64;
            let mut ipa: u64 = guest.address().as_u64() >> 12;
            unsafe {
                ipa = bits_in_reg(TLBI_OP::NS, tlbi_ns::IPAS_S)
                    | bits_in_reg(TLBI_OP::TTL, 0b0100 | _level)
                    | bits_in_reg(TLBI_OP::IPA, ipa);
                // corresponds to __kvm_tlb_flush_vmid_ipa()
                asm!(
                    "dsb ishst",
                    "tlbi ipas2e1is, {}",
                    "isb",
                    in(reg) ipa,
                );
            }
        }
    }
}

impl<'a> IPATranslation for Stage2Translation<'a> {
    fn get_base_address(&self) -> *const c_void {
        self.root_pgtlb as *const _ as *const c_void
    }

    fn set_pages(
        &mut self,
        guest: GuestPhysAddr,
        phys: PhysAddr,
        size: usize,
        flags: usize,
        is_raw: bool,
    ) -> Result<(), Error> {
        let _guest = Page::<BasePageSize, GuestPhysAddr>::range_with_size(guest, size);
        let _guest_copy = Page::<BasePageSize, GuestPhysAddr>::range_with_size(guest, size);
        let phys = Page::<BasePageSize, PhysAddr>::range_with_size(phys, size);

        if is_raw {
            if self
                .root_pgtlb
                .set_pages(_guest, phys, flags as u64, is_raw)
                .is_err()
            {
                warn!("set_pages error");
                return Err(Error::RmiErrorInput);
            }
        } else {
            if self
                .root_pgtlb
                .set_pages(
                    _guest,
                    phys,
                    flags as u64 | BasePageSize::MAP_EXTRA_FLAG,
                    is_raw,
                )
                .is_err()
            {
                warn!("set_pages error");
                return Err(Error::RmiErrorInput);
            }
        }
        self.tlb_flush_by_vmid_ipa::<BasePageSize>(_guest_copy);

        //TODO Set dirty only if pages are updated, not added
        self.dirty = true;
        Ok(())
    }

    fn unset_pages(&mut self, _guest: GuestPhysAddr, _size: usize) {
        //TODO implement
    }

    /// Retrieves Page Table Entry (PA) from Intermediate Physical Address (IPA)
    ///
    /// (input)
    ///   guest: a target guest physical address to translate
    ///   level: the intended page-table level to reach
    ///
    /// (output)
    ///   if exists,
    ///      physical address
    ///   else,
    ///      None
    fn ipa_to_pa(&mut self, guest: GuestPhysAddr, level: usize) -> Option<PhysAddr> {
        let guest = Page::<BasePageSize, GuestPhysAddr>::including_address(guest);
        let mut pa = None;
        let res = self.root_pgtlb.entry(guest, level, false, |entry| {
            pa = entry.address(0);
            Ok(None)
        });
        if res.is_ok() {
            pa
        } else {
            None
        }
    }

    /// Retrieves Page Table Entry (PTE) from Intermediate Physical Address (IPA)
    ///
    /// (input)
    ///   guest: a target guest physical address to translate
    ///   level: the intended page-table level to reach
    ///
    /// (output)
    ///   if exists,
    ///      A tuple of (pte value (u64), lastly reached page table level (usize))
    ///   else,
    ///      None
    fn ipa_to_pte(&mut self, guest: GuestPhysAddr, level: usize) -> Option<(u64, usize)> {
        let guest = Page::<BasePageSize, GuestPhysAddr>::including_address(guest);
        let mut pte = 0;
        let res = self.root_pgtlb.entry(guest, level, true, |entry| {
            pte = entry.pte();
            Ok(None)
        });
        if let Ok(x) = res {
            return Some((pte, x.1));
        } else {
            return None;
        }
    }

    fn clean(&mut self) {
        if self.dirty {
            unsafe {
                // According to DDI0608A E1.2.1.11 Cache and TLB operations
                // second half part
                asm! {
                    "
                    dsb ishst
                    tlbi vmalle1is
                    dsb ish
                    isb
                    "
                }
            }

            self.dirty = false;
        }
    }
}

impl<'a> fmt::Debug for Stage2Translation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(Self)).finish()
    }
}

impl<'a> Drop for Stage2Translation<'a> {
    fn drop(&mut self) {
        info!("drop Stage2Translation");
        self.root_pgtlb.drop();
    }
}

fn fill_stage2_table(
    root: &mut PageTable<GuestPhysAddr, L1Table, entry::Entry, { L1Table::NUM_ENTRIES }>,
) {
    let device_flags = helper::bits_in_reg(RawPTE::ATTR, pte::attribute::DEVICE_NGNRE)
        | helper::bits_in_reg(RawPTE::S2AP, pte::permission::RW);
    let uart_guest = Page::<BasePageSize, GuestPhysAddr>::range_with_size(
        GuestPhysAddr::from(0x1c0a0000 as u64),
        1,
    );
    let uart_phys =
        Page::<BasePageSize, PhysAddr>::range_with_size(PhysAddr::from(0x1c0a0000 as u64), 1);

    if root
        .set_pages(uart_guest, uart_phys, device_flags as u64, false)
        .is_err()
    {
        warn!("set_pages error");
    }
}
