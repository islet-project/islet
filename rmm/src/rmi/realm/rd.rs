use crate::rmi::rtt::realm_par_size;

use vmsa::guard::Content;

// TODO: Integrate with our `struct Realm`
#[derive(Debug)]
pub struct Rd {
    realm_id: usize,
    state: State,
    rtt_base: usize,
    ipa_bits: usize,
    rec_index: usize,
    s2_starting_level: isize,
    hash_algo: u8,
}

impl Rd {
    pub fn init(&mut self, id: usize, rtt_base: usize, ipa_bits: usize, s2_starting_level: isize) {
        self.realm_id = id;
        self.state = State::New;
        self.rtt_base = rtt_base;
        self.ipa_bits = ipa_bits;
        self.rec_index = 0;
        self.s2_starting_level = s2_starting_level;
    }

    pub fn id(&self) -> usize {
        self.realm_id
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
