pub mod handlers;
mod params;
pub mod run;

pub use self::handlers::set_event_handler;

use crate::mm::guard::Content;
use crate::rmi::realm::rd::State;
use crate::rmi::realm::Rd;
use crate::rmm::granule::GranuleState;

pub struct Rec {
    pub rd: Rd,
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
    pub fn init(&mut self, rd_id: usize, rd_state: State, vcpuid: usize) {
        self.rd.init_with_state(rd_id, rd_state); // Copy Rd into Rec space
        self.vcpuid = vcpuid;
        self.set_ripas(0, 0, 0, 0);
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

    pub fn ripas_addr(&mut self) -> u64 {
        self.ripas.addr
    }

    pub fn ripas_state(&self) -> u8 {
        self.ripas.state
    }

    pub fn ripas_end(&self) -> u64 {
        self.ripas.end
    }
}

impl Content for Rec {
    const FLAGS: u64 = GranuleState::Rec;
}
