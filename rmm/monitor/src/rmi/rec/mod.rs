pub mod handlers;
mod params;
pub mod run;

pub use self::handlers::set_event_handler;

use crate::rmi::realm::Rd;

use core::mem::ManuallyDrop;

pub struct Rec {
    pub rd: &'static Rd,
    vcpuid: usize,
    ripas: Ripas,
}

struct Ripas {
    start: u64,
    end: u64,
    addr: u64,
    state: u8,
}

impl Rec {
    pub unsafe fn new(
        rec_addr: usize,
        vcpuid: usize,
        rd: &'static Rd,
    ) -> ManuallyDrop<&'static mut Rec> {
        let rec: &mut Rec = &mut *(rec_addr as *mut Rec);
        rec.vcpuid = vcpuid;
        rec.rd = rd;
        ManuallyDrop::new(rec)
    }

    pub unsafe fn into(rec_addr: usize) -> ManuallyDrop<&'static mut Rec> {
        let rec: &mut Rec = &mut *(rec_addr as *mut Rec);
        ManuallyDrop::new(rec)
    }

    pub fn id(&self) -> usize {
        self.vcpuid
    }

    pub fn set_ripas(&mut self, start: u64, end: u64, addr: u64, state: u8) {
        self.ripas.start = start;
        self.ripas.end = end;
        self.ripas.addr = addr;
        self.ripas.state = state;
    }

    pub fn inc_ripas_addr(&mut self, size: u64) {
        self.ripas.addr += size;
    }

    pub fn ripas_addr(&mut self) -> usize {
        self.ripas.addr as usize
    }
}

impl Drop for Rec {
    fn drop(&mut self) {}
}
