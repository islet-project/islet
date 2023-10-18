pub mod handlers;
pub mod mpidr;
pub mod params;
pub mod run;
pub mod vtcr;

pub use self::handlers::set_event_handler;

use crate::granule::GranuleState;

use vmsa::guard::Content;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RmmRecAttestState {
    AttestInProgress,
    NoAttestInProgress,
}

pub struct Rec {
    attest_state: RmmRecAttestState,
    attest_challenge: [u8; 64],
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

    pub fn attest_state(&self) -> RmmRecAttestState {
        self.attest_state
    }

    pub fn attest_challenge(&self) -> &[u8] {
        &self.attest_challenge
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

    pub fn set_attest_state(&mut self, state: RmmRecAttestState) {
        self.attest_state = state;
    }

    pub fn set_attest_challenge(&mut self, challenge: &[u8]) {
        self.attest_challenge.copy_from_slice(challenge);
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
