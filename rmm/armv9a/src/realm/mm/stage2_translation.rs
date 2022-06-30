use super::page::BasePageSize;
use super::page_table::{entry::Entry, L1Table};

use core::ffi::c_void;
use core::fmt;

use monitor::mm::address::PhysAddr;
use monitor::mm::page::{Page, PageIter, PageSize};
use monitor::mm::page_table::{PageTable, PageTableMethods};
use monitor::realm::mm::address::GuestPhysAddr;
use monitor::realm::mm::IPATranslation;

use crate::helper::bits_in_reg;
use crate::helper::regs::tcr_granule;
use crate::{define_bitfield, define_bits, define_mask};
use crate::helper;
use crate::realm::mm::page_table::pte;
use crate::realm::mm::translation_granule_4k::RawPTE;

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
    root_pgtlb: &'a mut PageTable<GuestPhysAddr, L1Table, Entry>,
    dirty: bool,
}

impl<'a> Stage2Translation<'a> {
    pub fn new() -> Self {
        let root_pgtlb = unsafe {
            &mut *PageTable::<GuestPhysAddr, L1Table, Entry>::new_with_align::<BasePageSize>(
                NUM_ROOT_PAGE,
                ALIGN_ROOT_PAGE,
            )
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
                ipa = bits_in_reg(TLBI_OP::NS, tlbi_ns::IPAS_NS)
                    | bits_in_reg(TLBI_OP::TTL, tcr_granule::G_4K | _level)
                    | bits_in_reg(TLBI_OP::IPA, ipa);
                // corresponds to __kvm_tlb_flush_vmid_ipa()
                llvm_asm! {
                    "
                    dsb ishst
                    tlbi ipas2e1, $0
                    dsb sy
                    isb
                    " : : "r"(ipa)
                };
            }
        }
    }
}

impl<'a> IPATranslation for Stage2Translation<'a> {
    fn get_base_address(&self) -> *const c_void {
        self.root_pgtlb as *const _ as *const c_void
    }

    fn set_pages(&mut self, guest: GuestPhysAddr, phys: PhysAddr, size: usize, flags: usize) {
        let _guest = Page::<BasePageSize, GuestPhysAddr>::range_with_size(guest, size);
        let _guest_copy = Page::<BasePageSize, GuestPhysAddr>::range_with_size(guest, size);
        let phys = Page::<BasePageSize, PhysAddr>::range_with_size(phys, size);

        self.root_pgtlb.set_pages(_guest, phys, flags as u64);
        self.tlb_flush_by_vmid_ipa::<BasePageSize>(_guest_copy);

        //TODO Set dirty only if pages are updated, not added
        self.dirty = true;
    }

    fn unset_pages(&mut self, _guest: GuestPhysAddr, _size: usize) {
        //TODO implement
    }

    fn clean(&mut self) {
        if self.dirty {
            unsafe {
                // According to DDI0608A E1.2.1.11 Cache and TLB operations
                // second half part
                llvm_asm! {
                    "
                    dsb ishst
                    tlbi vmalle1
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

fn fill_stage2_table(root: &mut PageTable<GuestPhysAddr, L1Table, Entry>) {

    let device_flags = helper::bits_in_reg(RawPTE::ATTR, pte::attribute::DEVICE_NGNRE)
        | helper::bits_in_reg(RawPTE::S2AP, pte::permission::RW);
    let uart_guest = Page::<BasePageSize, GuestPhysAddr>::range_with_size(GuestPhysAddr::from(0x1c0a0000 as u64), 1);
    let uart_phys = Page::<BasePageSize, PhysAddr>::range_with_size(PhysAddr::from(0x1c0a0000 as u64), 1);

    root.set_pages(uart_guest, uart_phys, device_flags as u64);
}
