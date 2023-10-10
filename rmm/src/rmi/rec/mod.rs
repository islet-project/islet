pub mod handlers;
pub mod mpidr;
pub mod params;
pub mod run;
pub mod vtcr;

pub use self::handlers::set_event_handler;

use crate::granule::GranuleState;
use crate::rmi::error::Error;
use crate::rmi::realm::Rd;

use vmsa::guard::Content;

pub struct Rec {
    /// PA of RD of Realm which owns this REC
    owner: usize,
    vcpuid: usize,
    runnable: bool,
    ripas: Ripas,
    vtcr: u64,
    host_call_pending: bool,
}

struct Ripas {
    start: u64,
    end: u64,
    addr: u64,
    state: u8,
}

impl Rec {
    pub fn init(&mut self, owner: usize, vcpuid: usize, flags: u64) {
        self.owner = owner;
        self.vcpuid = vcpuid;
        self.set_ripas(0, 0, 0, 0);
        self.set_runnable(flags);
    }

    pub fn ipa_bits(&self) -> Result<usize, Error> {
        let rd = get_granule_if!(self.owner(), GranuleState::RD)?;
        let rd = rd.content::<Rd>();
        Ok(rd.ipa_bits())
    }

    pub fn realm_id(&self) -> Result<usize, Error> {
        let rd = get_granule_if!(self.owner(), GranuleState::RD)?;
        let rd = rd.content::<Rd>();
        Ok(rd.id())
    }

    pub fn runnable(&self) -> bool {
        self.runnable
    }

    pub fn id(&self) -> usize {
        self.vcpuid
    }

    pub fn owner(&self) -> usize {
        self.owner
    }

    pub fn host_call_pending(&self) -> bool {
        self.host_call_pending
    }

    pub fn set_host_call_pending(&mut self, val: bool) {
        self.host_call_pending = val;
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

    fn set_runnable(&mut self, flags: u64) {
        const RUNNABLE_OFFSET: u64 = 1;
        self.runnable = match flags & RUNNABLE_OFFSET {
            0 => false,
            _ => true,
        }
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
