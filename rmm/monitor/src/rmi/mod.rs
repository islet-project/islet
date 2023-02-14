use alloc::rc::Rc;
use core::cell::RefCell;
use core::cmp::Ordering;

use crate::smc;

use crate::call;
use crate::communication::{self, Error, ErrorKind};

extern crate alloc;

pub mod gpt;
pub mod realm;
pub mod version;

/* requested defined in tf-a-tests: realm_payload_test.h */
/* RMI_FNUM_VERSION_REQ ~ RMI_FNUM_REALM_DESTROY */
const RMM_VERSION: usize = 0xc400_0150;
const RMM_GRANULE_DELEGATE: usize = 0xc400_0151;
const RMM_GRANULE_UNDELEGATE: usize = 0xc400_0152;
const RMM_REALM_CREATE: usize = 0xc400_0158;
const RMM_REALM_DESTROY: usize = 0xc400_0159;
const RMM_REALM_RUN: usize = 0xc400_0160;
const RMM_VCPU_CREATE: usize = 0xc400_0161;
const RMM_REALM_MAP_MEMORY: usize = 0xc400_0170;
const RMM_REALM_UNMAP_MEMORY: usize = 0xc400_0171;
const RMM_REALM_SET_REG: usize = 0xc400_0172;
const RMM_REALM_GET_REG: usize = 0xc400_0173;
const RMM_REQ_COMPLETE: usize = 0xc400_018f;

pub const RET_SUCCESS: usize = 0x101;
pub const RET_FAIL: usize = 0x100;
pub const RET_EXCEPTION_IRQ: usize = 0x0;
pub const RET_EXCEPTION_SERROR: usize = 0x1;
pub const RET_EXCEPTION_TRAP: usize = 0x2;
pub const RET_EXCEPTION_IL: usize = 0x3;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Code {
    Version,
    RequestComplete,
    GranuleDelegate,
    GranuleUndelegate,
    RealmCreate,
    RealmDestroy,
    RealmMapMemory,
    RealmUnmapMemory,
    RealmSetReg,
    RealmGetReg,
    RealmRun,
    VCPUCreate,
    Unknown(usize),
}

impl From<Code> for usize {
    fn from(origin: Code) -> Self {
        match origin {
            Code::Version => RMM_VERSION,
            Code::RequestComplete => RMM_REQ_COMPLETE,
            Code::GranuleDelegate => RMM_GRANULE_DELEGATE,
            Code::GranuleUndelegate => RMM_GRANULE_UNDELEGATE,
            Code::RealmCreate => RMM_REALM_CREATE,
            Code::RealmDestroy => RMM_REALM_DESTROY,
            Code::RealmMapMemory => RMM_REALM_MAP_MEMORY,
            Code::RealmUnmapMemory => RMM_REALM_UNMAP_MEMORY,
            Code::RealmSetReg => RMM_REALM_SET_REG,
            Code::RealmGetReg => RMM_REALM_GET_REG,
            Code::RealmRun => RMM_REALM_RUN,
            Code::VCPUCreate => RMM_VCPU_CREATE,
            Code::Unknown(remain) => remain,
        }
    }
}

impl From<usize> for Code {
    fn from(origin: usize) -> Self {
        match origin {
            RMM_VERSION => Code::Version,
            RMM_REQ_COMPLETE => Code::RequestComplete,
            RMM_GRANULE_DELEGATE => Code::GranuleDelegate,
            RMM_GRANULE_UNDELEGATE => Code::GranuleUndelegate,
            RMM_REALM_CREATE => Code::RealmCreate,
            RMM_REALM_DESTROY => Code::RealmDestroy,
            RMM_REALM_MAP_MEMORY => Code::RealmMapMemory,
            RMM_REALM_UNMAP_MEMORY => Code::RealmUnmapMemory,
            RMM_REALM_SET_REG => Code::RealmSetReg,
            RMM_REALM_GET_REG => Code::RealmGetReg,
            RMM_REALM_RUN => Code::RealmRun,
            RMM_VCPU_CREATE => Code::VCPUCreate,
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

pub type Argument = [usize; 7];
pub type Return = usize;
pub type Call = call::Context<Code, Argument, Return>;

pub struct Receiver {
    sender: Rc<Sender>,
}

impl Receiver {
    pub fn new() -> Self {
        Self {
            sender: Rc::new(Sender::new()),
        }
    }
}

impl communication::Receiver for Receiver {
    type Event = Call;

    fn recv(&self) -> Result<Call, Error> {
        let cmd = usize::from(Code::RequestComplete);
        let arg = self.sender.pop();
        let smc = smc::instance().ok_or(Error::new(ErrorKind::Unsupported))?;
        let ret = smc.call(cmd, arg);

        let cmd = ret[0];
        let mut arg = [0usize; 7];
        arg.clone_from_slice(&ret[1..8]);
        Ok(Call::new(Code::from(cmd), arg, self.sender.clone()))
    }
}

pub struct Sender {
    data: RefCell<(usize, [Return; 4])>,
}

impl Sender {
    const fn new() -> Self {
        Self {
            data: RefCell::new((0usize, [0usize; 4])),
        }
    }

    fn pop(&self) -> [Return; 4] {
        let mut d = self.data.borrow_mut();
        let ret = d.1;
        *d = (0usize, [0usize; 4]);
        ret
    }

    fn push(&self, data: usize) -> Result<(), Error> {
        let mut d = self.data.borrow_mut();
        let pos = d.0;
        if pos < 4 {
            d.1[pos] = data;
            d.0 += 1;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::StorageFull))
        }
    }
}

impl communication::Sender for Sender {
    type Event = Return;

    fn send(&self, event: Return) -> Result<(), Error> {
        self.push(event)
    }
}
