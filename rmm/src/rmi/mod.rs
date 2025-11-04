pub mod constraint;
pub mod error;
pub mod features;
pub mod gpt;
pub mod metadata;
pub mod realm;
pub mod rec;
pub mod rtt;
pub mod version;

use crate::define_interface;

define_interface! {
    command {
         VERSION                = 0xc400_0150,
         GRANULE_DELEGATE       = 0xc400_0151,
         GRANULE_UNDELEGATE     = 0xc400_0152,
         DATA_CREATE            = 0xc400_0153,
         DATA_CREATE_UNKNOWN    = 0xc400_0154,
         DATA_DESTROY           = 0xc400_0155,
         REALM_ACTIVATE         = 0xc400_0157,
         REALM_CREATE           = 0xc400_0158,
         REALM_DESTROY          = 0xc400_0159,
         REC_CREATE             = 0xc400_015a,
         REC_DESTROY            = 0xc400_015b,
         REC_ENTER              = 0xc400_015c,
         RTT_CREATE             = 0xc400_015d,
         RTT_DESTROY            = 0xc400_015e,
         RTT_MAP_UNPROTECTED    = 0xc400_015f,
         RTT_UNMAP_UNPROTECTED  = 0xc400_0162,
         RTT_READ_ENTRY         = 0xc400_0161,
         PSCI_COMPLETE          = 0xc400_0164,
         FEATURES               = 0xc400_0165,
         RTT_FOLD               = 0xc400_0166,
         REC_AUX_COUNT          = 0xc400_0167,
         RTT_INIT_RIPAS         = 0xc400_0168,
         RTT_SET_RIPAS          = 0xc400_0169,
         // vendor calls
         ISLET_REALM_SET_METADATA = 0xc700_0150,
    }
}

pub const REQ_COMPLETE: usize = 0xc400_018f;

pub const RMM_GET_REALM_ATTEST_KEY: usize = 0xC400_01B2;
pub const RMM_GET_PLAT_TOKEN: usize = 0xC400_01B3;
pub const RMM_ISLET_GET_VHUK: usize = 0xC700_01B0;

pub const BOOT_COMPLETE: usize = 0xC400_01CF;
pub const BOOT_SUCCESS: usize = 0x0;

pub const NOT_SUPPORTED_YET: usize = 0xFFFF_EEEE;

pub const ABI_MAJOR_VERSION: usize = 1;
pub const ABI_MINOR_VERSION: usize = 0;

pub const HASH_ALGO_SHA256: u8 = 0;
pub const HASH_ALGO_SHA512: u8 = 1;

pub const RET_FAIL: usize = 0x100;
pub const RET_EXCEPTION_IRQ: usize = 0x0;
pub const RET_EXCEPTION_SERROR: usize = 0x1;
pub const RET_EXCEPTION_TRAP: usize = 0x2;
pub const RET_EXCEPTION_IL: usize = 0x3;

pub const SUCCESS: usize = 0;
pub const ERROR_INPUT: usize = 1;
pub const ERROR_REC: usize = 3;
pub const SUCCESS_REC_ENTER: usize = 4;

pub const PMU_OVERFLOW_NOT_ACTIVE: u8 = 0;
pub const PMU_OVERFLOW_ACTIVE: u8 = 1;

// RmiRttEntryState represents the state of an RTTE
pub mod rtt_entry_state {
    pub const RMI_UNASSIGNED: usize = 0;
    pub const RMI_ASSIGNED: usize = 1;
    pub const RMI_TABLE: usize = 2;
}

pub const MAX_REC_AUX_GRANULES: usize = 16;

pub const EXIT_SYNC: u8 = 0;
pub const EXIT_IRQ: u8 = 1;
pub const EXIT_FIQ: u8 = 2;
pub const EXIT_PSCI: u8 = 3;
pub const EXIT_RIPAS_CHANGE: u8 = 4;
pub const EXIT_HOST_CALL: u8 = 5;
pub const EXIT_SERROR: u8 = 6;
