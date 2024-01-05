use super::page::BasePageSize;
#[cfg(not(feature = "realm_linux"))]
use super::page_table::{entry, L0Table};
#[cfg(feature = "realm_linux")]
use super::page_table::{entry, L2Table};

use core::arch::asm;
use core::ffi::c_void;
use core::fmt;

use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::translation_granule_4k::RawPTE;
use crate::realm::mm::IPATranslation;
use crate::rmi::error::Error;
use alloc::alloc::Layout;
use vmsa::address::PhysAddr;
use vmsa::page::{Page, PageIter, PageSize};
use vmsa::page_table::Entry;
use vmsa::page_table::{Level, MemAlloc, PageTable, PageTableMethods};

use armv9a::{bits_in_reg, define_bitfield, define_bits, define_mask};

// initial lookup starts at level 1 with 2 page tables concatenated
pub const NUM_ROOT_PAGE: usize = 2;
pub const ALIGN_ROOT_PAGE: usize = 2;

pub mod tlbi_ns {
    pub const IPAS_S: u64 = 0b0;
    pub const IPAS_NS: u64 = 0b1;
}

define_bits!(TLBI_OP, NS[63 - 63], TTL[47 - 44], IPA[35 - 0]);

#[cfg(feature = "realm_linux")]
pub struct Stage2Translation<'a> {
    // We will set the translation granule with 4KB.
    root_pgtlb: &'a mut PageTable<
        GuestPhysAddr,
        L2Table,
        entry::Entry,
        { <L2Table as Level>::NUM_ENTRIES },
    >,
    dirty: bool,
}
#[cfg(not(feature = "realm_linux"))]
pub struct Stage2Translation<'a> {
    // We will set the translation granule with 4KB.
    root_pgtlb: &'a mut PageTable<
        GuestPhysAddr,
        L0Table,
        entry::Entry,
        { <L0Table as Level>::NUM_ENTRIES },
    >,
    dirty: bool,
}

impl<'a> Stage2Translation<'a> {
    #[cfg(feature = "realm_linux")]
    pub fn new(rtt_base: usize) -> Self {
        let root_pgtlb = unsafe {
            &mut *PageTable::<
                GuestPhysAddr,
                L2Table,
                entry::Entry,
                { <L2Table as Level>::NUM_ENTRIES },
            >::new_with_base(rtt_base)
            .unwrap()
        };

        Self {
            root_pgtlb,
            dirty: false,
        }
    }

    #[cfg(not(feature = "realm_linux"))]
    pub fn new(rtt_base: usize) -> Self {
        let root_pgtlb = unsafe {
            &mut *PageTable::<
                GuestPhysAddr,
                L0Table,
                entry::Entry,
                { <L0Table as Level>::NUM_ENTRIES },
            >::new_with_base(rtt_base)
            .unwrap()
        };

        Self {
            root_pgtlb,
            dirty: false,
        }
    }

    // According to DDI0608A E1.2.1.11 Cache and TLB operations
    // 'TLBI IPAS2E1, Xt; DSB; TLBI VMALLE1'
    // or TLBI ALL or TLBI VMALLS1S2
    #[allow(unused)]
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

impl<'a> MemAlloc for Stage2Translation<'a> {
    unsafe fn alloc(layout: Layout) -> *mut u8 {
        error!("alloc for Stage2Translation is not allowed. {:?}", layout);
        // Safety: the caller must do proper error handling with this null pointer.
        core::ptr::null_mut()
    }

    unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
        error!(
            "alloc_zeroed for Stage2Translation is not allowed. {:?}",
            layout
        );
        // Safety: the caller must do proper error handling with this null pointer.
        core::ptr::null_mut()
    }

    unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
        error!(
            "dealloc for Stage2Translation is not allowed. {:?}, {:?}",
            ptr, layout
        );
    }
}

impl<'a> IPATranslation for Stage2Translation<'a> {
    fn get_base_address(&self) -> *const c_void {
        self.root_pgtlb as *const _ as *const c_void
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

    fn ipa_to_pte_set(
        &mut self,
        guest: GuestPhysAddr,
        level: usize,
        val: u64,
    ) -> Result<(), Error> {
        let guest = Page::<BasePageSize, GuestPhysAddr>::including_address(guest);
        let res = self.root_pgtlb.entry(guest, level, true, |entry| {
            let pte = entry.mut_pte();
            *pte = RawPTE(val);
            Ok(None)
        });
        if let Ok(_x) = res {
            return Ok(());
        } else {
            return Err(Error::RmiErrorInput);
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
