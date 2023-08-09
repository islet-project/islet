use crate::mm::guard::Content;
use crate::rmm::granule::GranuleState;

pub struct Rd {
    realm_id: usize,
    state: State,
}

impl Rd {
    pub fn init(&mut self, id: usize) {
        self.realm_id = id;
        self.state = State::New;
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
