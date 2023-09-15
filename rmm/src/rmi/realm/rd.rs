use crate::granule::GranuleState;

use vmsa::guard::Content;

// TODO: Integrate with our `struct Realm`
pub struct Rd {
    realm_id: usize,
    state: State,
    rtt_base: usize,
    ipa_bits: usize,
}

impl Rd {
    pub fn init(&mut self, id: usize, rtt_base: usize, ipa_bits: usize) {
        self.realm_id = id;
        self.state = State::New;
        self.rtt_base = rtt_base;
        self.ipa_bits = ipa_bits;
    }

    pub fn init_with_state(&mut self, id: usize, state: State) {
        self.realm_id = id;
        self.state = state;
    }

    pub fn id(&self) -> usize {
        self.realm_id
    }

    pub fn state(&self) -> State {
        self.state
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
}

impl Content for Rd {
    const FLAGS: u64 = GranuleState::RD;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Null,
    New,
    Active,
    SystemOff,
}
