use core::arch::asm;
use core::ffi::c_void;
use core::fmt;
use core::slice::Iter;

use crate::granule::GRANULE_SHIFT;
use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::entry;
use crate::realm::mm::page::BasePageSize;
use crate::realm::mm::page::{HugePageSize, LargePageSize};
use crate::realm::mm::stage2_tte::mapping_size;
use crate::realm::mm::table_level::RootTable;
use crate::realm::mm::table_level::{L0Table, L1Table, L2Table};
use crate::realm::mm::IPATranslation;
use crate::rmi::error::Error;
use alloc::alloc::Layout;
use vmsa::address::PhysAddr;
use vmsa::page::{Page, PageIter, PageSize};
use vmsa::page_table::Entry;
use vmsa::page_table::{Level, MemAlloc, PageTable, PageTableMethods};

use aarch64_cpu::registers::*;
use armv9a::{bits_in_reg, define_bitfield, define_bits, define_mask};

pub mod tlbi_ns {
    pub const IPAS_S: u64 = 0b0;
    pub const IPAS_NS: u64 = 0b1;
}

pub enum Tlbi {
    NONE,
    LEAF(usize),      // vmid : usize
    BREAKDOWN(usize), // vmid: usize
}

define_bits!(TLBI_OP, NS[63 - 63], TTL[47 - 44], IPA[35 - 0]);
define_bits!(
    TLBI_RANGE_OP,
    NS[63 - 63],
    TG[47 - 46],
    SCALE[45 - 44],
    NUM[43 - 39],
    TTL[38 - 37],
    IPA[36 - 0]
);

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
        >::new_init_in(&RttAllocator { base: $base }, |entries| {
            for e in entries.iter_mut() {
                *e = Entry::new();
            }
        })
        .unwrap()
    };
}

pub struct Stage2Translation<'a> {
    // We will set the translation granule with 4KB.
    root_pgtbl: Root<'a>,
    root_level: usize,
    root_pages: usize,
    dirty: bool,
}

impl Stage2Translation<'_> {
    pub fn new(rtt_base: usize, root_level: usize, root_pages: usize) -> Self {
        // Concatenated translation tables
        // For stage 2 address translations, for the initial lookup,
        // up to 16 translation tables can be concatenated.
        let root_pgtbl = match root_level {
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
            root_pgtbl,
            root_level,
            root_pages,
            dirty: false,
        }
    }

    fn tlbi_iter<S: PageSize>(level: usize, guest_iter: PageIter<S, GuestPhysAddr>, vmid: usize) {
        let vmid_saved = VTTBR_EL2.read(VTTBR_EL2::VMID);
        VTTBR_EL2.write(VTTBR_EL2::VMID.val(vmid as u64));

        unsafe {
            asm!("isb");
        }
        for guest in guest_iter {
            let mut ipa: u64 = guest.address().as_u64() >> GRANULE_SHIFT;
            unsafe {
                ipa = bits_in_reg(TLBI_OP::TTL, 0b0100 | level as u64)
                    | bits_in_reg(TLBI_OP::IPA, ipa);
                asm!(
                    "tlbi IPAS2E1IS, {}",
                    in(reg) ipa,
                );
            }
        }
        unsafe {
            asm!("dsb ish");
        }

        VTTBR_EL2.write(VTTBR_EL2::VMID.val(vmid_saved));
    }

    // According to DDI0608A E1.2.1.11 Cache and TLB operations
    // 'TLBI IPAS2E1, Xt; DSB; TLBI VMALLE1'
    // or TLBI ALL or TLBI VMALLS1S2
    fn tlbi_by_vmid_ipa(level: usize, guest: GuestPhysAddr, vmid: usize) {
        let guest_iter =
            Page::<BasePageSize, GuestPhysAddr>::range_with_size(guest, BasePageSize::SIZE);
        Self::tlbi_iter(level, guest_iter, vmid);
        // TODO: Do it at once when switchng the context to the realm
        Self::tlbi_vmalle1is(vmid);
    }

    #[allow(unused)]
    // Saved in case tlb-rmi arch extension is not provided
    fn tlbi_by_vmid_ipa_range_v2(level: usize, guest: GuestPhysAddr, vmid: usize) {
        match level {
            L2Table::THIS_LEVEL => {
                let guest_iter = Page::<BasePageSize, GuestPhysAddr>::range_with_size(
                    guest,
                    mapping_size(level),
                );
                Self::tlbi_iter(level + 1, guest_iter, vmid);
            }
            L1Table::THIS_LEVEL => {
                let guest_iter = Page::<LargePageSize, GuestPhysAddr>::range_with_size(
                    guest,
                    mapping_size(level),
                );
                Self::tlbi_iter(level + 1, guest_iter, vmid);
            }
            L0Table::THIS_LEVEL => {
                let guest_iter = Page::<HugePageSize, GuestPhysAddr>::range_with_size(
                    guest,
                    mapping_size(level),
                );
                Self::tlbi_iter(level + 1, guest_iter, vmid);
            }
            _ => {
                panic!("wrong invalidation level")
            }
        };
        // TODO: Do it at once when switchng the context to the realm
        Self::tlbi_vmalle1is(vmid);
    }

    fn tlbi_by_vmid_ipa_range(level: usize, guest: GuestPhysAddr, vmid: usize) {
        let vmid_saved = VTTBR_EL2.read(VTTBR_EL2::VMID);
        VTTBR_EL2.write(VTTBR_EL2::VMID.val(vmid as u64));
        unsafe {
            asm!("isb");
        }

        // The entry is within the address range determined by the formula
        // [BaseADDR <= VA < BaseADDR + ((NUM +1)*2(5*SCALE +1) * Translation_Granule_Size)].
        let scale = 3 - level as u64;
        let num = 8 + 4 * scale;
        unsafe {
            let xt = bits_in_reg(TLBI_RANGE_OP::TG, 0b01)    // 4K tranlsation granule
                | bits_in_reg(TLBI_RANGE_OP::SCALE, scale)
                | bits_in_reg(TLBI_RANGE_OP::NUM, num)
                | bits_in_reg(TLBI_RANGE_OP::TTL, (level + 1) as u64)
                | bits_in_reg(TLBI_RANGE_OP::IPA, guest.as_u64());
            asm!(
                "tlbi RIPAS2LE1, {}",
                in(reg) xt,
            );
        }
        unsafe {
            asm!("dsb ish");
        }
        VTTBR_EL2.write(VTTBR_EL2::VMID.val(vmid_saved));
        // TODO: Do it at once when switchng the context to the realm
        Self::tlbi_vmalle1is(vmid);
    }

    fn tlbi_vmalle1is(vmid: usize) {
        unsafe {
            let vmid_saved = VTTBR_EL2.read(VTTBR_EL2::VMID);
            VTTBR_EL2.write(VTTBR_EL2::VMID.val(vmid as u64));
            // According to DDI0608A E1.2.1.11 Cache and TLB operations
            // second half part
            asm! {
                "
                    dsb ishst
                    tlbi VMALLE1IS
                    dsb ish
                    isb
                    "
            }
            VTTBR_EL2.write(VTTBR_EL2::VMID.val(vmid_saved));
        }
    }
}

pub struct RttAllocator {
    pub base: usize,
}

impl MemAlloc for RttAllocator {
    unsafe fn allocate(&self, _layout: Layout) -> *mut u8 {
        //TODO: check alignment
        self.base as *mut u8
    }

    unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        error!(
            "dealloc for RttAllocator will do nothing. {:?}, {:?}",
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

// ipa_to_pte closure
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
            let _ = entry.set(PhysAddr::from(0 as u64), $val); //FIXME: get pa
            Ok(None)
        })
    };
}

impl IPATranslation for Stage2Translation<'_> {
    fn get_base_address(&self) -> *const c_void {
        match &self.root_pgtbl {
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

        let res = match &mut self.root_pgtbl {
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
        let res = match &mut self.root_pgtbl {
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
        invalidate: Tlbi,
    ) -> Result<(), Error> {
        let map_addr = guest;
        let guest = Page::<BasePageSize, GuestPhysAddr>::including_address(guest);
        let res = match &mut self.root_pgtbl {
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
            match invalidate {
                Tlbi::LEAF(vmid) => {
                    #[cfg(not(any(miri, test)))]
                    Self::tlbi_by_vmid_ipa(level, map_addr, vmid);
                    self.dirty = true;
                }
                Tlbi::BREAKDOWN(vmid) => {
                    #[cfg(not(any(miri, test)))]
                    Self::tlbi_by_vmid_ipa_range(level, map_addr, vmid);
                    self.dirty = true;
                }
                _ => {}
            }

            Ok(())
        } else {
            Err(Error::RmiErrorInput)
        }
    }

    fn clean(&mut self, vmid: usize) {
        if self.dirty {
            Self::tlbi_vmalle1is(vmid);
            self.dirty = false;
        }
    }

    fn space_size(&self, level: usize) -> usize {
        let count = if level == self.root_level {
            self.root_pages
        } else {
            1
        };
        if level == 0 {
            mapping_size(0) * L0Table::NUM_ENTRIES * count
        } else {
            mapping_size(level - 1) * count
        }
    }

    fn entries(
        &self,
        guest: GuestPhysAddr,
        level: usize,
    ) -> Result<(Iter<'_, entry::Entry>, usize), Error> {
        let guest = Page::<BasePageSize, GuestPhysAddr>::including_address(guest);
        let res = match &self.root_pgtbl {
            Root::L2N8(root) => root.table_entries(guest, level),
            Root::L0N1(root) => root.table_entries(guest, level),
            Root::L0N16(root) => root.table_entries(guest, level),
            Root::L1N1(root) => root.table_entries(guest, level),
            Root::L1N2(root) => root.table_entries(guest, level),
            Root::L1N8(root) => root.table_entries(guest, level),
            Root::L2N4(root) => root.table_entries(guest, level),
            Root::L2N16(root) => root.table_entries(guest, level),
        };
        if let Ok(ref _x) = res {
            Ok(res?)
        } else {
            Err(Error::RmiErrorInput)
        }
    }
}

impl fmt::Debug for Stage2Translation<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(Self)).finish()
    }
}
