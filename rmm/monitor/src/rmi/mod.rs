use crate::error::Error;

pub mod constraint;
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
pub const DATA_CREATE_UNKNOWN: usize = 0xc400_0154;
pub const DATA_DESTORY: usize = 0xc400_0155;
pub const REALM_ACTIVATE: usize = 0xc400_0157;
pub const REALM_CREATE: usize = 0xc400_0158;
pub const REALM_DESTROY: usize = 0xc400_0159;
pub const REC_CREATE: usize = 0xc400_015a;
pub const REC_DESTROY: usize = 0xc400_015b;
pub const REC_ENTER: usize = 0xc400_015c;
pub const RTT_MAP_UNPROTECTED: usize = 0xc400_015f;
pub const RTT_READ_ENTRY: usize = 0xc400_0161;
pub const FEATURES: usize = 0xc400_0165;
pub const REC_AUX_COUNT: usize = 0xc400_0167;
pub const RTT_INIT_RIPAS: usize = 0xc400_0168;
pub const RTT_SET_RIPAS: usize = 0xc400_0169;
pub const REQ_COMPLETE: usize = 0xc400_018f;

pub const BOOT_COMPLETE: usize = 0xC400_01CF;
pub const BOOT_SUCCESS: usize = 0x0;

pub const ABI_MAJOR_VERSION: usize = 1;
pub const ABI_MINOR_VERSION: usize = 0;

pub const RET_FAIL: usize = 0x100;
pub const RET_EXCEPTION_IRQ: usize = 0x0;
pub const RET_EXCEPTION_SERROR: usize = 0x1;
pub const RET_EXCEPTION_TRAP: usize = 0x2;
pub const RET_EXCEPTION_IL: usize = 0x3;

pub const SUCCESS: usize = 0;
pub const ERROR_INPUT: usize = 1;
pub const ERROR_REC: usize = 3;
pub const SUCCESS_REC_ENTER: usize = 4;

pub const MAX_REC_AUX_GRANULES: usize = 16;

pub const EXIT_SYNC: u8 = 0;
pub const EXIT_IRQ: u8 = 1;
pub const EXIT_FIQ: u8 = 2;
pub const EXIT_PSCI: u8 = 3;
pub const EXIT_RIPAS_CHANGE: u8 = 4;
pub const EXIT_HOST_CALL: u8 = 5;
pub const EXIT_SERROR: u8 = 6;

pub type RMI = &'static dyn Interface;

pub struct MapProt(usize);

impl From<usize> for MapProt {
    fn from(prot: usize) -> Self {
        Self(prot as usize)
    }
}

impl MapProt {
    pub fn new(data: usize) -> Self {
        MapProt(data)
    }
    pub fn set_bit(&mut self, prot: u64) {
        self.0 |= 1 << prot;
    }
    pub fn get(&self) -> usize {
        self.0
    }
    pub fn is_set(&self, prot: u64) -> bool {
        (self.0 >> prot) & 1 == 1
    }
    pub const DEVICE: u64 = 0;
    pub const NS_PAS: u64 = 1;
}

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
