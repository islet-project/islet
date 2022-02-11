use core::cmp::Ordering;

use crate::smc;

use realm_management_monitor::call;
use realm_management_monitor::communication::{self, Error};

const RMM_VERSION: usize = 0xc000_0000;
const RMM_REQ_COMPLETE: usize = 0xc000_0010;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Code {
    Version,
    RequestComplete,
    Unknown(usize),
}

impl From<Code> for usize {
    fn from(origin: Code) -> Self {
        match origin {
            Code::Version => RMM_VERSION,
            Code::RequestComplete => RMM_REQ_COMPLETE,
            Code::Unknown(remain) => remain,
        }
    }
}

impl From<usize> for Code {
    fn from(origin: usize) -> Self {
        match origin {
            RMM_VERSION => Code::Version,
            RMM_REQ_COMPLETE => Code::RequestComplete,
            remain => Code::Unknown(remain),
        }
    }
}

impl Default for Code {
    fn default() -> Self {
        Code::Unknown(0)
    }
}

impl PartialOrd for Code {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Code {
    fn cmp(&self, other: &Self) -> Ordering {
        usize::from(*self).cmp(&usize::from(*other))
    }
}

pub struct Receiver;

impl Receiver {
    pub const fn new() -> Self {
        Self {}
    }
}

pub type Argument = [usize; 4];
pub type Call = call::Context<Code, Argument>;

impl communication::Receiver for Receiver {
    type Event = Call;

    fn recv(&self) -> Result<Call, Error> {
        let ret = smc::call([usize::from(Code::RequestComplete), 0, 0, 0, 0]);

        let code = ret[0];
        let mut args = [0usize; 4];
        args.copy_from_slice(&ret[1..5]);

        Ok(Call::new(Code::from(code), args))
    }
}
