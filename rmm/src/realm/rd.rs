use vmsa::guard::Content;

use crate::measurement::{Measurement, MEASUREMENTS_SLOT_NR};
use crate::realm::mm::IPATranslation;
use crate::simd::SimdConfig;
use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use spin::mutex::Mutex;

pub const RPV_SIZE: usize = 64;

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
    rtt_num_start: usize,
    ipa_bits: usize,
    rec_index: usize,
    s2_starting_level: isize,
    hash_algo: u8,
    rpv: [u8; RPV_SIZE],
    pub measurements: [Measurement; MEASUREMENTS_SLOT_NR],
    pub vcpu_index: usize,
    metadata: Option<usize>,
    simd_cfg: SimdConfig,
}

impl Rd {
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        &mut self,
        vmid: u16,
        rtt_base: usize,
        rtt_num_start: usize,
        ipa_bits: usize,
        s2_starting_level: isize,
        rpv: [u8; 64],
        sve_en: bool,
        sve_vl: u64,
    ) {
        self.vmid = vmid;
        self.state = State::New;
        self.rtt_base = rtt_base;
        self.rtt_num_start = rtt_num_start;
        self.ipa_bits = ipa_bits;
        self.rec_index = 0;
        self.s2_starting_level = s2_starting_level;
        self.measurements = [Measurement::empty(); MEASUREMENTS_SLOT_NR];
        self.vcpu_index = 0;
        self.rpv.copy_from_slice(rpv.as_slice());
        self.metadata = None;
        self.simd_cfg.sve_en = sve_en;
        if sve_en {
            self.simd_cfg.sve_vq = sve_vl;
        }
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

    pub fn rtt_num_start(&self) -> usize {
        self.rtt_num_start
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

    pub fn ipa_size(&self) -> usize {
        1 << self.ipa_bits
    }

    pub fn par_size(&self) -> usize {
        self.ipa_size() / 2
    }

    pub fn addr_in_par(&self, ipa: usize) -> bool {
        ipa < self.par_size()
    }

    pub fn hash_algo(&self) -> u8 {
        self.hash_algo
    }

    pub fn set_hash_algo(&mut self, alg: u8) {
        self.hash_algo = alg;
    }

    pub fn personalization_value(&self) -> &[u8] {
        self.rpv.as_slice()
    }

    pub fn metadata(&self) -> Option<usize> {
        self.metadata
    }

    pub fn set_metadata(&mut self, metadata: Option<usize>) {
        self.metadata = metadata
    }

    pub fn simd_config(&self) -> &SimdConfig {
        &self.simd_cfg
    }
}

impl Content for Rd {}

impl safe_abstraction::raw_ptr::RawPtr for Rd {}

impl safe_abstraction::raw_ptr::SafetyChecked for Rd {}

impl safe_abstraction::raw_ptr::SafetyAssured for Rd {
    fn is_initialized(&self) -> bool {
        // The initialization of this memory is guaranteed
        // according to the RMM Specification A2.2.4 Granule Wiping.
        // This instance belongs to a RD Granule and has been initialized.
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Null,
    New,
    Active,
    SystemOff,
}
