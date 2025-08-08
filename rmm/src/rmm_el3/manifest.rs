use autopadding::*;
use safe_abstraction::raw_ptr::assume_safe;
use safe_abstraction::raw_ptr::Error;
use spinning_top::SpinlockGuard;

use super::RMM_SHARED_BUFFER_LOCK;
use crate::config;
/*
 * Boot Manifest structure illustration, with two dram banks and
 * a single console.
 *
 * +----------------------------------------+
 * | offset |     field      |    comment   |
 * +--------+----------------+--------------+
 * |   0    |    version     |  0x00000003  |
 * +--------+----------------+--------------+
 * |   4    |    padding     |  0x00000000  |
 * +--------+----------------+--------------+
 * |   8    |   plat_data    |     NULL     |
 * +--------+----------------+--------------+
 * |   16   |   num_banks    |              |
 * +--------+----------------+              |
 * |   24   |     banks      |   plat_dram  |
 * +--------+----------------+              |
 * |   32   |    checksum    |              |
 * +--------+----------------+--------------+
 * |   40   |  num_consoles  |              |
 * +--------+----------------+              |
 * |   48   |    consoles    | plat_console |
 * +--------+----------------+              |
 * |   56   |    checksum    |              |
 * +--------+----------------+--------------+
 * |   64   |     base 0     |              |
 * +--------+----------------+    bank[0]   |
 * |   72   |     size 0     |              |
 * +--------+----------------+--------------+
 * +--------+----------------+    bank[N]   |
 * +--------+----------------+--------------+
 * |   X    |     base       |              |
 * +--------+----------------+              |
 * |   X+8  |   map_pages    |              |
 * +--------+----------------+              |
 * |   X+16 |     name       |              |
 * +--------+----------------+  consoles[0] |
 * |   X+24 |   clk_in_hz    |              |
 * +--------+----------------+              |
 * |   X+32 |   baud_rate    |              |
 * +--------+----------------+              |
 * |   X+40 |     flags      |              |
 * +--------+----------------+--------------+
 * +--------+----------------+  consoles[M] |
 * +--------+----------------+--------------+
 */

// Console info structure
/*
#[repr(C)]
pub struct ConsoleInfo {
    pub base: u64,       // Console base address
    pub map_pages: u64,  // Num of pages to be mapped in RMM for the console MMIO
    pub name: [u8; 8],           // Name of console
    pub clk_in_hz: u64,  // UART clock (in Hz) for the console
    pub baud_rate: u64,  // Baud rate
    pub flags: u64,      // Additional flags RES0
}
*/

// NS DRAM bank structure
#[repr(C)]
pub struct DramBank {
    pub base: u64, // Base address
    pub size: u64, // Size of bank
}

// Boot manifest core structure as per v0.3
pad_struct_and_impl_default!(
pub struct RmmManifest {
    0x0  pub version: u32,           // Manifest version
    0x8  pub plat_data_ptr: u64,     // Manifest platform data
    0x10 pub num_banks: u64,         // plat_dram.banks
    0x18 pub banks_ptr: u64,         // plat_dram.banks_ptr
    0x20 pub banks_checksum: u64,     // plat_dram.bank_checksum
    0x28 pub num_consoles: u64,      // plat_console.num_consoles
    0x30 pub consoles_ptr: u64,      // plat_console.consoles
    0x38 pub console_checksum: u64,  // plat_console.checksum
    0x40 => @END,
}
);

const EL3_IFC_VERSION: u32 = 0x00000003;

pub fn load() -> core::result::Result<(), Error> {
    debug!("Configuring RMM with EL3 manifest");
    let guard: SpinlockGuard<'_, _> = RMM_SHARED_BUFFER_LOCK.lock();
    let manifest = assume_safe::<RmmManifest>(*guard)?;
    let struct_size = core::mem::size_of::<DramBank>();
    let mut dram_vec = config::NS_DRAM_REGIONS.lock();
    debug!("version: {:?}", manifest.version);
    debug!("num_banks: {:x}", manifest.num_banks);
    if manifest.version != EL3_IFC_VERSION {
        panic!(
            "manifest version {:X} not supported. requires {:X}",
            manifest.version, EL3_IFC_VERSION
        );
    }
    let mut bank_ptr = manifest.banks_ptr as usize;
    let mut max_base = 0;
    for i in 0..manifest.num_banks {
        let bank = assume_safe::<DramBank>(bank_ptr)?;
        debug!(
            "NS_DRAM[{:?}]: {:X}-{:X} (size:{:X})",
            i,
            bank.base,
            bank.base + bank.size,
            bank.size
        );
        dram_vec.push(core::ops::Range {
            start: bank.base as usize,
            end: (bank.base + bank.size) as usize,
        });
        bank_ptr += struct_size;
        if bank.base < max_base {
            panic!("Islet only accepts an ordered bank list");
        }
        max_base = bank.base;
    }
    Ok(())
}

impl safe_abstraction::raw_ptr::RawPtr for RmmManifest {}

impl safe_abstraction::raw_ptr::SafetyChecked for RmmManifest {}

impl safe_abstraction::raw_ptr::SafetyAssured for RmmManifest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn verify_ownership(&self) -> bool {
        true
    }
}

impl safe_abstraction::raw_ptr::RawPtr for DramBank {}

impl safe_abstraction::raw_ptr::SafetyChecked for DramBank {}

impl safe_abstraction::raw_ptr::SafetyAssured for DramBank {
    fn is_initialized(&self) -> bool {
        true
    }

    fn verify_ownership(&self) -> bool {
        true
    }
}
