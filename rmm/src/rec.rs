use crate::gic;
use crate::realm::context::Context;
use crate::realm::rd::Rd;
use crate::realm::timer;
use crate::rmi::error::Error;
use crate::rmi::rec::params::NR_AUX;
use crate::rmm_exit;
use crate::rsi::attestation::MAX_CHALLENGE_SIZE;

use aarch64_cpu::registers::*;

use core::cell::OnceCell;
use vmsa::guard::Content;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RmmRecAttestState {
    AttestInProgress,
    NoAttestInProgress,
}

#[derive(Copy, Clone, Debug)]
pub enum State {
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

#[repr(C)]
#[derive(Debug)]
pub struct Rec<'a> {
    pub context: Context,
    attest_state: RmmRecAttestState,
    // TODO: Create consts for both numbers
    attest_challenge: [u8; MAX_CHALLENGE_SIZE],
    attest_token_offset: usize,
    aux: [u64; NR_AUX], // Addresses of auxiliary Granules
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
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            attest_state: RmmRecAttestState::NoAttestInProgress,
            attest_challenge: [0; MAX_CHALLENGE_SIZE],
            attest_token_offset: 0,
            aux: [0; NR_AUX],
            owner: OnceCell::new(),
            vcpuid: 0,
            runnable: false,
            psci_pending: false,
            state: State::Ready,
            ripas: Ripas {
                start: 0,
                end: 0,
                addr: 0,
                state: 0,
                flags: 0,
            },
            vtcr: 0,
            host_call_pending: false,
        }
    }

    pub fn init(
        &mut self,
        owner: usize,
        vcpuid: usize,
        flags: u64,
        aux: [u64; NR_AUX],
        vttbr: u64,
        vmpidr: u64,
    ) -> Result<(), Error> {
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
        self.set_runnable(flags);
        self.context.sys_regs.vttbr = vttbr;
        self.context.sys_regs.vmpidr = vmpidr;
        self.aux.copy_from_slice(&aux);
        timer::init_timer(self);
        gic::init_gic(self);

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

    pub fn from_current(&mut self) {
        unsafe {
            Context::from_current(self);
        }
    }

    pub fn into_current(&self) {
        unsafe {
            Context::into_current(self);
        }
    }

    pub fn reset_ctx(&mut self) {
        self.context.spsr = (SPSR_EL2::D.mask << SPSR_EL2::D.shift)
            | (SPSR_EL2::A.mask << SPSR_EL2::A.shift)
            | (SPSR_EL2::I.mask << SPSR_EL2::I.shift)
            | (SPSR_EL2::F.mask << SPSR_EL2::F.shift)
            | (SPSR_EL2::M.mask & u64::from(SPSR_EL2::M::EL1h)) << SPSR_EL2::M.shift;

        self.context.sys_regs.sctlr = 0;
    }
}

impl Content for Rec<'_> {}

impl safe_abstraction::raw_ptr::RawPtr for Rec<'_> {}

impl safe_abstraction::raw_ptr::SafetyChecked for Rec<'_> {}

impl safe_abstraction::raw_ptr::SafetyAssured for Rec<'_> {
    fn is_initialized(&self) -> bool {
        // The initialization of this memory is guaranteed
        // according to the RMM Specification A2.2.4 Granule Wiping.
        // This instance belongs to a REC Granule and has been initialized.
        true
    }

    fn verify_ownership(&self) -> bool {
        // The ownership of this instance is exclusively ensured by the RMM.
        // under the following conditions:
        //
        // 1. A lock on the given address is obtained using the `get_granule*` macros.
        // 2. The instance is converted from a raw pointer through the `content*` functions.
        // 3. The instance is accessed only within the lock scope.
        //
        // Ownership verification is guaranteed because these criteria are satisfied
        // in all cases where this object is accessed.
        true
    }
}

// XXX: is using 'static okay here?
unsafe fn current() -> Option<&'static mut Rec<'static>> {
    match TPIDR_EL2.get() {
        0 => None,
        current => Some(&mut *(current as *mut Rec<'_>)),
    }
}

fn enter() -> [usize; 4] {
    unsafe {
        if let Some(_rec) = current() {
            // TODO: add code equivalent to the previous clean()
            return rmm_exit([0; 4]);
        }
        [0, 0, 0, 0]
    }
}

fn exit() {
    unsafe {
        if let Some(rec) = current() {
            rec.from_current();
        }
    }
}

// TODO: check the below again
pub fn run_prepare(rd: &Rd, vcpu: usize, rec: &mut Rec<'_>, incr_pc: usize) -> Result<(), Error> {
    if incr_pc == 1 {
        rec.context.elr += 4;
    }
    debug!("resuming: {:#x}", rec.context.elr);
    rec.into_current();

    trace!("Switched to VCPU {} on Realm {}", vcpu, rd.id());
    Ok(())
}

pub fn run() -> Result<[usize; 4], Error> {
    let ret = enter();

    exit();
    Ok(ret)
}
