use alloc::rc::Rc;
use core::cell::RefCell;
use core::cmp::Ordering;

use crate::smc;

use monitor::call;
use monitor::communication::{self, Error, ErrorKind};

extern crate alloc;

pub mod gpt;
pub mod realm;
pub mod version;

const RMM_VERSION: usize = 0xc000_0000;
const RMM_GRANULE_DELEGATE: usize = 0xc000_0001;
const RMM_GRANULE_UNDELEGATE: usize = 0xc000_0002;
const RMM_VM_CREATE: usize = 0xc000_0003;
const RMM_VM_SWITCH: usize = 0xc000_0004;
const RMM_VM_DESTROY: usize = 0xc000_0006;
const RMM_VM_MAP_MEMORY: usize = 0xc000_0007;
const RMM_VM_UNMAP_MEMORY: usize = 0xc000_0008;
const RMM_VM_SET_REG: usize = 0xc000_0009;
const RMM_VM_GET_REG: usize = 0xc000_000a;
const RMM_VM_RUN: usize = 0xc000_000b;
const RMM_VCPU_CREATE: usize = 0xc000_000c;
const RMM_REQ_COMPLETE: usize = 0xc000_0010;

pub const RET_SUCCESS: usize = 0x0;
pub const RET_PAGE_FAULT: usize = 0x1;
pub const RET_FAIL: usize = 0x100;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Code {
    Version,
    RequestComplete,
    GranuleDelegate,
    GranuleUndelegate,
    VMCreate,
    VMSwitch,
    VMDestroy,
    VMMapMemory,
    VMUnmapMemory,
    VMSetReg,
    VMGetReg,
    VMRun,
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
            Code::VMCreate => RMM_VM_CREATE,
            Code::VMSwitch => RMM_VM_SWITCH,
            Code::VMDestroy => RMM_VM_DESTROY,
            Code::VMMapMemory => RMM_VM_MAP_MEMORY,
            Code::VMUnmapMemory => RMM_VM_UNMAP_MEMORY,
            Code::VMSetReg => RMM_VM_SET_REG,
            Code::VMGetReg => RMM_VM_GET_REG,
            Code::VMRun => RMM_VM_RUN,
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
            RMM_VM_CREATE => Code::VMCreate,
            RMM_VM_SWITCH => Code::VMSwitch,
            RMM_VM_DESTROY => Code::VMDestroy,
            RMM_VM_MAP_MEMORY => Code::VMMapMemory,
            RMM_VM_UNMAP_MEMORY => Code::VMUnmapMemory,
            RMM_VM_SET_REG => Code::VMSetReg,
            RMM_VM_GET_REG => Code::VMGetReg,
            RMM_VM_RUN => Code::VMRun,
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

pub type Argument = [usize; 4];
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
        let ret = smc::call(cmd, arg);

        let cmd = ret[0];
        let mut arg = [0usize; 4];
        arg.clone_from_slice(&ret[1..5]);
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
