use super::page::BasePageSize;
use super::page_table::{entry, RootTable};
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

const ENTR1: usize = <RootTable<0, 1> as Level>::NUM_ENTRIES;
const ENTR2: usize = ENTR1 * 2;
const ENTR4: usize = ENTR1 * 4;
const ENTR8: usize = ENTR1 * 8;
const ENTR16: usize = ENTR1 * 16;

type RootTBL<'a, const L: usize, const N: usize, const E: usize> =
    &'a mut PageTable<GuestPhysAddr, RootTable<{ L }, { N }>, entry::Entry, { E }>;

pub enum Root<'a> {
    L0N1(RootTBL<'a, 0, 1, ENTR1>),
    L0N16(RootTBL<'a, 0, 16, ENTR16>),
    L1N1(RootTBL<'a, 1, 1, ENTR1>),
    L1N2(RootTBL<'a, 1, 2, ENTR2>),
    L1N8(RootTBL<'a, 1, 8, ENTR8>),
    L2N4(RootTBL<'a, 2, 4, ENTR4>),
    L2N8(RootTBL<'a, 2, 8, ENTR8>),
    L2N16(RootTBL<'a, 2, 16, ENTR16>),
}

#[macro_export]
macro_rules! init_table {
    ($level:expr, $pages:expr, $base:expr) => {
        &mut *PageTable::<
            GuestPhysAddr,
            RootTable<$level, $pages>,
            entry::Entry,
            { <RootTable<$level, $pages> as Level>::NUM_ENTRIES },
        >::new_with_base($base)
        .unwrap()
    };
}

pub struct Stage2Translation<'a> {
    // We will set the translation granule with 4KB.
    root_pgtlb: Root<'a>,
    //root_level: usize,
    //root_pages: usize,
    dirty: bool,
}

impl<'a> Stage2Translation<'a> {
    pub fn new(rtt_base: usize, root_level: usize, root_pages: usize) -> Self {
        // Concatenated translation tables
        // For stage 2 address translations, for the initial lookup,
        // up to 16 translation tables can be concatenated.
        let root_pgtlb = match root_level {
            0 => unsafe {
                match root_pages {
                    1 => Root::L0N1(init_table!(0, 1, rtt_base)),
                    16 => Root::L0N16(init_table!(0, 16, rtt_base)),
                    _ => todo!(),
                }
            },
            1 => unsafe {
                match root_pages {
                    1 => Root::L1N1(init_table!(1, 1, rtt_base)),
                    2 => Root::L1N2(init_table!(1, 2, rtt_base)),
                    8 => Root::L1N8(init_table!(1, 8, rtt_base)),
                    _ => todo!(),
                }
            },
            2 => unsafe {
                match root_pages {
                    4 => Root::L2N4(init_table!(2, 4, rtt_base)),
                    8 => Root::L2N8(init_table!(2, 8, rtt_base)),
                    16 => Root::L2N16(init_table!(2, 16, rtt_base)),
                    _ => todo!(),
                }
            },
            _ => todo!(),
        };
        Self {
            root_pgtlb,
            //root_level,
            //root_pages,
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

#[macro_export]
// ipa_to_pa closure
macro_rules! to_pa {
    ($root:expr, $guest:expr, $level:expr, $pa:expr) => {
        $root.entry($guest, $level, false, |entry| {
            $pa = entry.address(0);
            Ok(None)
        })
    };
}

// ipa_to_pa closure
macro_rules! to_pte {
    ($root:expr, $guest:expr, $level:expr, $pte:expr) => {
        $root.entry($guest, $level, true, |entry| {
            $pte = entry.pte();
            Ok(None)
        })
    };
}

// ipa_to_pte_set clousre
macro_rules! set_pte {
    ($root:expr, $guest:expr, $level:expr, $val:expr) => {
        $root.entry($guest, $level, true, |entry| {
            let pte = entry.mut_pte();
            *pte = RawPTE($val);
            Ok(None)
        })
    };
}

impl<'a> IPATranslation for Stage2Translation<'a> {
    fn get_base_address(&self) -> *const c_void {
        match &self.root_pgtlb {
            Root::L2N8(c) => *c as *const _ as *const c_void, // most likely first, for linux-realm
            Root::L0N1(a) => *a as *const _ as *const c_void,
            Root::L0N16(a) => *a as *const _ as *const c_void,
            Root::L1N1(b) => *b as *const _ as *const c_void,
            Root::L1N2(b) => *b as *const _ as *const c_void,
            Root::L1N8(b) => *b as *const _ as *const c_void,
            Root::L2N4(c) => *c as *const _ as *const c_void,
            Root::L2N16(c) => *c as *const _ as *const c_void,
        }
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

        let res = match &mut self.root_pgtlb {
            Root::L2N8(root) => to_pa!(root, guest, level, pa), // most likely first, for linux-realm
            Root::L0N1(root) => to_pa!(root, guest, level, pa),
            Root::L0N16(root) => to_pa!(root, guest, level, pa),
            Root::L1N1(root) => to_pa!(root, guest, level, pa),
            Root::L1N2(root) => to_pa!(root, guest, level, pa),
            Root::L1N8(root) => to_pa!(root, guest, level, pa),
            Root::L2N4(root) => to_pa!(root, guest, level, pa),
            Root::L2N16(root) => to_pa!(root, guest, level, pa),
        };
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
        let res = match &mut self.root_pgtlb {
            Root::L2N8(root) => to_pte!(root, guest, level, pte), // most likely first, for linux-realm
            Root::L0N1(root) => to_pte!(root, guest, level, pte),
            Root::L0N16(root) => to_pte!(root, guest, level, pte),
            Root::L1N1(root) => to_pte!(root, guest, level, pte),
            Root::L1N2(root) => to_pte!(root, guest, level, pte),
            Root::L1N8(root) => to_pte!(root, guest, level, pte),
            Root::L2N4(root) => to_pte!(root, guest, level, pte),
            Root::L2N16(root) => to_pte!(root, guest, level, pte),
        };
        if let Ok(x) = res {
            Some((pte, x.1))
        } else {
            None
        }
    }

    fn ipa_to_pte_set(
        &mut self,
        guest: GuestPhysAddr,
        level: usize,
        val: u64,
    ) -> Result<(), Error> {
        let guest = Page::<BasePageSize, GuestPhysAddr>::including_address(guest);
        let res = match &mut self.root_pgtlb {
            Root::L2N8(root) => set_pte!(root, guest, level, val),
            Root::L0N1(root) => set_pte!(root, guest, level, val),
            Root::L0N16(root) => set_pte!(root, guest, level, val),
            Root::L1N1(root) => set_pte!(root, guest, level, val),
            Root::L1N2(root) => set_pte!(root, guest, level, val),
            Root::L1N8(root) => set_pte!(root, guest, level, val),
            Root::L2N4(root) => set_pte!(root, guest, level, val),
            Root::L2N16(root) => set_pte!(root, guest, level, val),
        };
        if let Ok(_x) = res {
            Ok(())
        } else {
            Err(Error::RmiErrorInput)
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
