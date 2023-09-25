pub mod handlers;
pub mod mpidr;
mod params;
pub mod run;
pub mod vtcr;

pub use self::handlers::set_event_handler;

use crate::granule::GranuleState;
use crate::rmi::error::Error;
use crate::rmi::realm::Rd;

use vmsa::guard::Content;

pub struct Rec {
    pub rd: Rd,
    /// PA of RD of Realm which owns this REC
    owner: usize,
    vcpuid: usize,
    ripas: Ripas,
    vtcr: u64,
}

struct Ripas {
    start: u64,
    end: u64,
    addr: u64,
    state: u8,
}

impl Rec {
    pub fn init(&mut self, owner: usize, vcpuid: usize) {
        self.owner = owner;
        self.vcpuid = vcpuid;
        self.set_ripas(0, 0, 0, 0);
    }

    pub fn ipa_bits(&self) -> Result<usize, Error> {
        let rd = get_granule_if!(self.owner(), GranuleState::RD)?;
        let rd = rd.content::<Rd>();
        Ok(rd.ipa_bits())
    }

    pub fn id(&self) -> usize {
        self.vcpuid
    }

    pub fn owner(&self) -> usize {
        self.owner
    }

    pub fn set_ripas(&mut self, start: u64, end: u64, addr: u64, state: u8) {
        self.ripas.start = start;
        self.ripas.end = end;
        self.ripas.addr = addr;
        self.ripas.state = state;
    }

    pub fn set_vtcr(&mut self, vtcr: u64) {
        self.vtcr = vtcr;
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

    pub fn vtcr(&self) -> u64 {
        self.vtcr
    }
}

impl Content for Rec {
    const FLAGS: u64 = GranuleState::Rec;
}
