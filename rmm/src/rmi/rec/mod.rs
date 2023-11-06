pub mod exit;
pub mod handlers;
pub mod mpidr;
pub mod params;
pub mod run;
pub mod vtcr;
use crate::rmi::error::Error;
use core::cell::OnceCell;

pub use self::handlers::set_event_handler;

use crate::granule::GranuleState;

use vmsa::guard::Content;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RmmRecAttestState {
    AttestInProgress,
    NoAttestInProgress,
}

struct Ripas {
    start: u64,
    end: u64,
    addr: u64,
    state: u8,
}

#[derive(Debug)]
struct Inner {
    owner: usize,
    realmid: usize,
    ipa_bits: usize,
}

impl Inner {
    pub fn init(owner: usize, realmid: usize, ipa_bits: usize) -> Self {
        Inner {
            owner,
            realmid,
            ipa_bits,
        }
    }

    pub fn owner(&self) -> usize {
        self.owner
    }

    pub fn realmid(&self) -> usize {
        self.realmid
    }

    pub fn ipa_bits(&self) -> usize {
        self.ipa_bits
    }
}

/// Immutable Realm information in target Rd.
///
/// It can be written to only once because of OnceCell.
/// Multiple RECs can be created per Realm
/// so the fields in Inner are always valid
/// before destroying the current Rec.
struct ImmutRealmInfo {
    inner: OnceCell<Inner>,
}

impl ImmutRealmInfo {
    fn init(&mut self, owner: usize, realmid: usize, ipa_bits: usize) -> Result<(), Inner> {
        self.inner.set(Inner::init(owner, realmid, ipa_bits))
    }

    fn get(&self) -> Result<&Inner, Error> {
        match self.inner.get() {
            Some(realm_info) => Ok(realm_info),
            None => {
                error!("ImmutRealmInfo is none");
                Err(Error::RmiErrorRec)
            }
        }
    }

    pub fn owner(&self) -> Result<usize, Error> {
        let inner = self.get()?;
        Ok(inner.owner())
    }

    pub fn realmid(&self) -> Result<usize, Error> {
        let inner = self.get()?;
        Ok(inner.realmid())
    }

    pub fn ipa_bits(&self) -> Result<usize, Error> {
        let inner = self.get()?;
        Ok(inner.ipa_bits())
    }
}

pub struct Rec {
    attest_state: RmmRecAttestState,
    attest_challenge: [u8; 64],
    /// PA of RD of Realm which owns this REC
    vcpuid: usize,
    runnable: bool,
    ripas: Ripas,
    vtcr: u64,
    host_call_pending: bool,
    realm_info: ImmutRealmInfo,
}

impl Rec {
    pub fn init(
        &mut self,
        owner: usize,
        vcpuid: usize,
        flags: u64,
        realmid: usize,
        ipa_bits: usize,
    ) -> Result<(), Error> {
        if let Err(new_realm_info) = self.realm_info.init(owner, realmid, ipa_bits) {
            let cur_realm_info = self.realm_info.get()?;
            error!(
                "Rec::init() called twice. cur_realm_info: {:?}, new_realm_info: {:?}",
                cur_realm_info, new_realm_info
            );
            return Err(Error::RmiErrorRec);
        }

        self.vcpuid = vcpuid;
        self.set_ripas(0, 0, 0, 0);
        self.set_runnable(flags);

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

    pub fn owner(&self) -> Result<usize, Error> {
        self.realm_info.owner()
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

    pub fn realmid(&self) -> Result<usize, Error> {
        self.realm_info.realmid()
    }

    pub fn ipa_bits(&self) -> Result<usize, Error> {
        self.realm_info.ipa_bits()
    }
}

impl Content for Rec {
    const FLAGS: u64 = GranuleState::Rec;
}
