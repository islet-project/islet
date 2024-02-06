pub mod exit;
pub mod handlers;
pub mod mpidr;
pub mod params;
pub mod run;
pub mod vtcr;
use crate::realm;
use crate::realm::registry::get_realm;
use crate::realm::vcpu::State as RecState;
use crate::realm::vcpu::VCPU;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;
use crate::rmi::Rd;
use crate::rmm_exit;
use core::cell::OnceCell;

pub use self::handlers::set_event_handler;

use vmsa::guard::Content;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RmmRecAttestState {
    AttestInProgress,
    NoAttestInProgress,
}

#[derive(Debug)]
struct Ripas {
    start: u64,
    end: u64,
    addr: u64,
    state: u8,
}

#[derive(Debug)]
pub struct Rec<'a> {
    attest_state: RmmRecAttestState,
    attest_challenge: [u8; 64],
    /// PA of RD of Realm which owns this REC
    ///
    /// Safety:
    /// Only immutable fields of Rd must be dereferenced by owner
    /// by making getter method for the safety
    owner: OnceCell<&'a Rd>,
    vcpuid: usize,
    runnable: bool,
    state: RecState,
    ripas: Ripas,
    vtcr: u64,
    host_call_pending: bool,
}

impl Rec<'_> {
    pub fn init(&mut self, owner: usize, vcpuid: usize, flags: u64) -> Result<(), Error> {
        if owner == 0 {
            error!("owner should be non-zero");
            return Err(Error::RmiErrorInput);
        }

        if let Err(input_owner) = self.owner.set(unsafe { &*(owner as *const Rd) }) {
            error!(
                "Rec::init() called twice. cur owner: {:x}, input owner: {:x}",
                self.get_owner()? as *const Rd as usize,
                input_owner as *const Rd as usize
            );
            return Err(Error::RmiErrorRec);
        }

        self.vcpuid = vcpuid;
        self.set_ripas(0, 0, 0, 0);
        self.set_runnable(flags);
        self.set_state(RecState::Ready);

        Ok(())
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

    pub fn vcpuid(&self) -> usize {
        self.vcpuid
    }

    fn get_owner(&self) -> Result<&Rd, Error> {
        match self.owner.get() {
            Some(owner) => Ok(owner),
            None => Err(Error::RmiErrorRec),
        }
    }

    pub fn owner(&self) -> Result<usize, Error> {
        let owner = self.get_owner()?;
        Ok(owner as *const Rd as usize)
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

    pub fn set_state(&mut self, state: RecState) {
        self.state = state;
    }

    pub fn get_state(&self) -> RecState {
        self.state
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

    pub fn realmid(&self) -> Result<usize, Error> {
        let owner = self.get_owner()?;
        Ok(owner.id())
    }

    pub fn ipa_bits(&self) -> Result<usize, Error> {
        let owner = self.get_owner()?;
        Ok(owner.ipa_bits())
    }
}

impl Content for Rec<'_> {}

fn enter() -> [usize; 4] {
    unsafe {
        if let Some(vcpu) = realm::vcpu::current() {
            if vcpu.is_realm_dead() {
                vcpu.from_current();
            } else {
                vcpu.realm.lock().page_table.lock().clean();
                return rmm_exit([0; 4]);
            }
        }
        [0, 0, 0, 0]
    }
}

fn exit() {
    unsafe {
        if let Some(vcpu) = realm::vcpu::current() {
            vcpu.from_current();
        }
    }
}

pub fn run(id: usize, vcpu: usize, incr_pc: usize) -> Result<[usize; 4], Error> {
    if incr_pc == 1 {
        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .vcpus
            .get(vcpu)
            .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
            .lock()
            .context
            .elr += 4;
    }
    debug!(
        "resuming: {:#x}",
        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .vcpus
            .get(vcpu)
            .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
            .lock()
            .context
            .elr
    );

    if let Some(vcpu) = get_realm(id)
        .ok_or(Error::RmiErrorOthers(NotExistRealm))?
        .lock()
        .vcpus
        .get(vcpu)
    {
        VCPU::into_current(&mut *vcpu.lock())
    }

    trace!("Switched to VCPU {} on Realm {}", vcpu, id);
    let ret = enter();

    exit();
    Ok(ret)
}
