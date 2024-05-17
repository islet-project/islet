use crate::rmi::rtt::realm_par_size;

use vmsa::guard::Content;

use crate::measurement::{Measurement, MEASUREMENTS_SLOT_NR};
use crate::realm::mm::stage2_translation::Stage2Translation;
use crate::realm::mm::IPATranslation;
use crate::realm::vcpu::VCPU;
use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::mutex::Mutex;

lazy_static! {
    static ref RTT_TABLES: Mutex<BTreeMap<usize, Arc<Mutex<Box<dyn IPATranslation>>>>> = {
        let m = BTreeMap::new();
        Mutex::new(m)
    };
}

pub fn insert_rtt(id: usize, table: Arc<Mutex<Box<dyn IPATranslation>>>) {
    RTT_TABLES.lock().insert(id, table);
}

#[derive(Debug)]
pub struct Rd {
    vmid: u16,
    state: State,
    rtt_base: usize,
    ipa_bits: usize,
    rec_index: usize,
    s2_starting_level: isize,
    hash_algo: u8,
    pub measurements: [Measurement; MEASUREMENTS_SLOT_NR],
    pub vcpus: Vec<Arc<Mutex<VCPU>>>,
}

impl Rd {
    pub fn init(&mut self, vmid: u16, rtt_base: usize, ipa_bits: usize, s2_starting_level: isize) {
        self.vmid = vmid;
        self.state = State::New;
        self.rtt_base = rtt_base;
        self.ipa_bits = ipa_bits;
        self.rec_index = 0;
        self.s2_starting_level = s2_starting_level;
        self.measurements = [Measurement::empty(); MEASUREMENTS_SLOT_NR];
        self.vcpus = Vec::new();
    }

    pub fn id(&self) -> usize {
        self.vmid as usize
    }

    pub fn s2_table(&self) -> Arc<Mutex<Box<dyn IPATranslation>>> {
        Arc::clone(RTT_TABLES.lock().get_mut(&self.id()).unwrap())
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
}

impl Content for Rd {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Null,
    New,
    Active,
    SystemOff,
}
