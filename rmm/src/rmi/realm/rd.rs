use crate::rmi::rtt::realm_par_size;

use vmsa::guard::Content;

use crate::measurement::{Measurement, MEASUREMENTS_SLOT_NR};
use crate::realm::mm::IPATranslation;
use crate::realm::vcpu::VCPU;
use crate::rmi::error::{Error, Error::*};
use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::mutex::Mutex;

#[derive(Debug)]
pub struct Rd {
    vmid: u16,
    state: State,
    rtt_base: usize,
    ipa_bits: usize,
    rec_index: usize,
    s2_starting_level: isize,
    s2_table: Arc<Mutex<Box<dyn IPATranslation>>>,
    hash_algo: u8,
    pub measurements: [Measurement; MEASUREMENTS_SLOT_NR],
    pub vcpus: Vec<Arc<Mutex<VCPU>>>,
    shrm_token: [Option<u8>; 2],
}

impl Rd {
    pub fn init(
        &mut self,
        vmid: u16,
        rtt_base: usize,
        ipa_bits: usize,
        s2_starting_level: isize,
        s2_table: Arc<Mutex<Box<dyn IPATranslation>>>,
    ) {
        self.vmid = vmid;
        self.state = State::New;
        self.rtt_base = rtt_base;
        self.ipa_bits = ipa_bits;
        self.rec_index = 0;
        self.s2_starting_level = s2_starting_level;
        // XXX: without `clone()`, the below assignment would cause a data abort exception
        self.s2_table = s2_table.clone();
        self.measurements = [Measurement::empty(); MEASUREMENTS_SLOT_NR];
        self.vcpus = Vec::new();
        self.shrm_token = [None; 2];
    }

    pub fn id(&self) -> usize {
        self.vmid as usize
    }

    pub fn s2_table(&self) -> &Arc<Mutex<Box<dyn IPATranslation>>> {
        &self.s2_table
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn at_state(&self, compared: State) -> bool {
        self.state == compared
    }

    pub fn rtt_base(&self) -> usize {
        self.rtt_base
    }

    pub fn ipa_bits(&self) -> usize {
        self.ipa_bits
    }

    pub fn rec_index(&self) -> usize {
        self.rec_index
    }

    pub fn s2_starting_level(&self) -> isize {
        self.s2_starting_level
    }

    pub fn inc_rec_index(&mut self) {
        self.rec_index += 1;
    }

    pub fn addr_in_par(&self, addr: usize) -> bool {
        let ipa_bits = self.ipa_bits();
        addr < realm_par_size(ipa_bits)
    }

    pub fn hash_algo(&self) -> u8 {
        self.hash_algo
    }

    pub fn set_hash_algo(&mut self, alg: u8) {
        self.hash_algo = alg;
    }

    pub fn set_shrm_token(&mut self, idx: usize, token: Option<u8>) -> Result<(), Error> {
        if 2 <= idx {
            error!("invalid shrm token idx {}", idx);
            return Err(RmiErrorInput);
        }
        self.shrm_token[idx] = token;
        Ok(())
    }

    pub fn shrm_token(&self, idx: usize) -> Result<Option<u8>, Error> {
        if 2 <= idx {
            error!("invalid shrm token idx {}", idx);
            return Err(RmiErrorInput);
        }

        Ok(self.shrm_token[idx])
    }
}

impl Content for Rd {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Null,
    New,
    Active,
    SystemOff,
}
