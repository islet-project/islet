use crate::error::Error;

pub mod features;
pub mod gpt;
pub mod realm;
pub mod rec;
pub mod rtt;
pub mod version;

pub const VERSION: usize = 0xc400_0150;
pub const GRANULE_DELEGATE: usize = 0xc400_0151;
pub const GRANULE_UNDELEGATE: usize = 0xc400_0152;
pub const DATA_CREATE: usize = 0xc400_0153;
pub const REALM_ACTIVATE: usize = 0xc400_0157;
pub const REALM_CREATE: usize = 0xc400_0158;
pub const REALM_DESTROY: usize = 0xc400_0159;
pub const REC_CREATE: usize = 0xc400_015a;
pub const REC_ENTER: usize = 0xc400_015c;
pub const RTT_MAP_UNPROTECTED: usize = 0xc400_015f;
pub const REALM_RUN: usize = 0xc400_0160;
pub const VCPU_CREATE: usize = 0xc400_0161;
pub const FEATURES: usize = 0xc400_0165;
pub const REC_AUX_COUNT: usize = 0xc400_0167;
pub const RTT_INIT_RIPAS: usize = 0xc400_0168;
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
pub const ERROR_REC: usize = 3;

pub const MAX_REC_AUX_GRANULES: usize = 16;

pub type RMI = &'static dyn Interface;

pub trait Interface {
    fn create_realm(&self) -> Result<usize, &str>;
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

pub(crate) fn dummy() {
    trace!("Dummy implementation.");
}
