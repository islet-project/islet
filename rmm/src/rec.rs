use crate::realm;
use crate::realm::rd::Rd;
use crate::realm::vcpu::VCPU;
use crate::rmi::error::Error;
use crate::rmi::error::InternalError::*;
use crate::rmm_exit;
use crate::rsi::attestation::MAX_CHALLENGE_SIZE;

use core::cell::OnceCell;
use vmsa::guard::Content;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RmmRecAttestState {
    AttestInProgress,
    NoAttestInProgress,
}

#[derive(Copy, Clone, Debug)]
pub enum State {
    Null = 0,
    Ready = 1,
    Running = 2,
}

#[derive(Debug)]
struct Ripas {
    start: u64,
    end: u64,
    addr: u64,
    state: u8,
    flags: u64,
}

#[derive(Debug)]
pub struct Rec<'a> {
    attest_state: RmmRecAttestState,
    // TODO: Create consts for both numbers
    attest_challenge: [u8; MAX_CHALLENGE_SIZE],
    attest_token_offset: usize,
    /// PA of RD of Realm which owns this REC
    ///
    /// Safety:
    /// Only immutable fields of Rd must be dereferenced by owner
    /// by making getter method for the safety
    owner: OnceCell<&'a Rd>,
    vcpuid: usize,
    runnable: bool,
    psci_pending: bool,
    state: State,
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
        self.set_state(State::Ready);

        Ok(())
    }

    pub fn attest_state(&self) -> RmmRecAttestState {
        self.attest_state
    }

    pub fn attest_challenge(&self) -> &[u8] {
        &self.attest_challenge
    }

    pub fn attest_token_offset(&self) -> usize {
        self.attest_token_offset
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

    pub fn psci_pending(&self) -> bool {
        self.psci_pending
    }

    pub fn set_attest_state(&mut self, state: RmmRecAttestState) {
        self.attest_state = state;
    }

    pub fn set_attest_challenge(&mut self, challenge: &[u8]) {
        self.attest_challenge.copy_from_slice(challenge);
    }

    pub fn set_attest_offset(&mut self, offset: usize) {
        self.attest_token_offset = offset;
    }

    pub fn set_host_call_pending(&mut self, val: bool) {
        self.host_call_pending = val;
    }

    pub fn set_psci_pending(&mut self, val: bool) {
        self.psci_pending = val;
    }

    pub fn set_ripas(&mut self, start: u64, end: u64, state: u8, flags: u64) {
        self.ripas.start = start;
        self.ripas.end = end;
        self.ripas.state = state;
        self.ripas.flags = flags;
        self.ripas.addr = start; // reset addr to the start
    }

    pub fn set_vtcr(&mut self, vtcr: u64) {
        self.vtcr = vtcr;
    }

    //TODO: change interface. A Rec state can be set by other Recs in the same Rd.
    pub fn set_runnable(&mut self, flags: u64) {
        const RUNNABLE_OFFSET: u64 = 1;
        self.runnable = match flags & RUNNABLE_OFFSET {
            0 => false,
            _ => true,
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn set_ripas_addr(&mut self, addr: u64) {
        self.ripas.addr = addr;
    }

    pub fn ripas_addr(&self) -> u64 {
        self.ripas.addr
    }

    pub fn ripas_start(&self) -> u64 {
        self.ripas.start
    }

    pub fn ripas_end(&self) -> u64 {
        self.ripas.end
    }

    pub fn ripas_state(&self) -> u8 {
        self.ripas.state
    }

    pub fn ripas_flags(&self) -> u64 {
        self.ripas.flags
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
                // TODO: add code equivalent to the previous clean()
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

pub fn run_prepare(rd: &Rd, vcpu: usize, incr_pc: usize) -> Result<(), Error> {
    if incr_pc == 1 {
        rd.vcpus
            .get(vcpu)
            .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
            .lock()
            .context
            .elr += 4;
    }
    debug!(
        "resuming: {:#x}",
        rd.vcpus
            .get(vcpu)
            .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
            .lock()
            .context
            .elr
    );
    if let Some(vcpu) = rd.vcpus.get(vcpu) {
        VCPU::into_current(&mut *vcpu.lock())
    }

    trace!("Switched to VCPU {} on Realm {}", vcpu, rd.id());
    Ok(())
}

pub fn run() -> Result<[usize; 4], Error> {
    let ret = enter();

    exit();
    Ok(ret)
}
