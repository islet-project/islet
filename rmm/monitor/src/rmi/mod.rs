use crate::error::Error;

pub mod features;
pub mod gpt;
pub mod realm;
pub mod version;

pub const VERSION: usize = 0xc400_0150;
pub const GRANULE_DELEGATE: usize = 0xc400_0151;
pub const GRANULE_UNDELEGATE: usize = 0xc400_0152;
pub const REALM_CREATE: usize = 0xc400_0158;
pub const REALM_DESTROY: usize = 0xc400_0159;
pub const REALM_RUN: usize = 0xc400_0160;
pub const VCPU_CREATE: usize = 0xc400_0161;
pub const FEATURES: usize = 0xc400_0165;
pub const REALM_MAP_MEMORY: usize = 0xc400_0170;
pub const REALM_UNMAP_MEMORY: usize = 0xc400_0171;
pub const REALM_SET_REG: usize = 0xc400_0172;
pub const REALM_GET_REG: usize = 0xc400_0173;
pub const REQ_COMPLETE: usize = 0xc400_018f;

pub const BOOT_COMPLETE: usize = 0xC400_01CF;
pub const BOOT_SUCCESS: usize = 0x0;

pub const ABI_VERSION: usize = 1;

pub const RET_SUCCESS: usize = 0x101;
pub const RET_FAIL: usize = 0x100;
pub const RET_EXCEPTION_IRQ: usize = 0x0;
pub const RET_EXCEPTION_SERROR: usize = 0x1;
pub const RET_EXCEPTION_TRAP: usize = 0x2;
pub const RET_EXCEPTION_IL: usize = 0x3;

pub const SUCCESS: usize = 0;
pub const ERROR_INPUT: usize = 1;

pub type RMI = &'static dyn Interface;

pub trait Interface {
    fn create(&self) -> Result<usize, &str>;
    fn create_vcpu(&self, id: usize) -> Result<usize, Error>;
    fn remove(&self, id: usize) -> Result<(), &str>;
    fn run(&self, id: usize, vcpu: usize, incr_pc: usize) -> Result<([usize; 4]), &str>;
    fn map(
        &self,
        id: usize,
        guest: usize,
        phys: usize,
        size: usize,
        prot: usize,
    ) -> Result<(), &str>;
    fn unmap(&self, id: usize, guest: usize, size: usize) -> Result<(), &str>;
    fn set_reg(&self, id: usize, vcpu: usize, register: usize, value: usize) -> Result<(), &str>;
    fn get_reg(&self, id: usize, vcpu: usize, register: usize) -> Result<usize, &str>;
}
