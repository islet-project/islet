use core::ffi::CStr;

use crate::granule::GranuleState;
use crate::measurement::{Measurement, MEASUREMENTS_SLOT_MAX_SIZE, MEASUREMENTS_SLOT_RIM};
use crate::realm::rd::{Rd, RPV_SIZE};
use crate::rmi::error::Error;
use crate::rmi::metadata::{IsletRealmMetadata, P384_PUBLIC_KEY_SIZE, REALM_ID_SIZE};
use crate::rmm_el3::{vhuk_a, vhuk_m};
use crate::{get_granule, get_granule_if};

const RSI_ISLET_USE_VHUK_M: usize = 0x1 << 0;
const RSI_ISLET_SLK_RIM: usize = 0x1 << 1;
const RSI_ISLET_SLK_REALM_ID: usize = 0x1 << 2;
const RSI_ISLET_SLK_SVN: usize = 0x1 << 3;

pub const SEALING_KEY_SIZE: usize = 32;

const SALT: [u8; 32] = [
    0xd5, 0x77, 0x5f, 0x52, 0x4a, 0xce, 0x32, 0x21, 0xce, 0x77, 0x1e, 0xd2, 0x74, 0xbb, 0x74, 0xa4,
    0x60, 0xce, 0x3f, 0xb9, 0x74, 0x9c, 0xe3, 0x7d, 0x0a, 0xe6, 0xd2, 0xe9, 0x07, 0xf8, 0xb5, 0x4b,
];

#[repr(C, packed)]
struct KdfInfo {
    public_key: [u8; P384_PUBLIC_KEY_SIZE],
    realm_id: [u8; REALM_ID_SIZE],
    rpv: [u8; RPV_SIZE],
    flags: usize,
    rim: [u8; MEASUREMENTS_SLOT_MAX_SIZE],
    hash_algo: u8,
    svn: usize,
}

impl KdfInfo {
    fn new() -> Self {
        Self {
            public_key: [0; P384_PUBLIC_KEY_SIZE],
            realm_id: [0; REALM_ID_SIZE],
            rpv: [0; RPV_SIZE],
            flags: 0,
            rim: [0; MEASUREMENTS_SLOT_MAX_SIZE],
            hash_algo: 0,
            svn: 0,
        }
    }

    // TODO: use zeroize?
    fn zeroize(&mut self) {
        let addr = self as *mut Self;
        unsafe {
            core::ptr::write_bytes(addr as *mut u8, 0x0, core::mem::size_of::<Self>());
        }
    }

    fn realm_id_as_str(&self) -> Option<&str> {
        let Ok(cstr) = CStr::from_bytes_until_nul(&self.realm_id) else {
            return None;
        };
        let Ok(s) = cstr.to_str() else {
            return None;
        };
        Some(s)
    }

    fn dump(&self) {
        info!("KDF info");
        info!("public_key: {}", hex::encode(self.public_key));
        info!(
            "realm_id: {}",
            self.realm_id_as_str().unwrap_or("INVALID REALM ID")
        );
        info!("rpv: {}", hex::encode(self.rpv));
        let flags = self.flags; // not aligned
        info!("flags: {:#010x}", flags);
        info!("rim: {}", hex::encode(self.rim));
        info!("hash_algo: {:#04x}", self.hash_algo);
        let svn = self.svn; // not aligned
        info!("svn: {:#010x}", svn);
    }

    fn as_u8_slice(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }

    fn derive_sealing_key(
        &self,
        use_vhuk_m: bool,
        okm: &mut [u8; SEALING_KEY_SIZE],
    ) -> core::result::Result<(), Error> {
        let ikm = if use_vhuk_m { vhuk_m() } else { vhuk_a() };
        let info = self.as_u8_slice();

        let hkdf = hkdf::Hkdf::<sha2::Sha256>::new(Some(&SALT), &ikm);
        hkdf.expand(&info, okm).or(Err(Error::RmiErrorInput))?;

        Ok(())
    }
}

pub fn realm_sealing_key(
    rd: &Rd,
    flags: usize,
    svn: usize,
    buf: &mut [u8; SEALING_KEY_SIZE],
) -> core::result::Result<(), Error> {
    info!("flags: {:#010x}, svn: {:#010x}", flags, svn);

    let mut info = KdfInfo::new();

    info.rpv.copy_from_slice(rd.personalization_value());
    info.flags = flags;

    if let Some(meta_addr) = rd.metadata() {
        let g_metadata = get_granule_if!(meta_addr, GranuleState::Metadata)?;
        let metadata = g_metadata.content::<IsletRealmMetadata>()?;

        if flags & RSI_ISLET_SLK_SVN != 0 && metadata.svn() < svn {
            warn!("The SVN parameter is invalid!");
            Err(Error::RmiErrorInput)?
        }

        info.public_key = metadata.public_key().clone();

        if flags & RSI_ISLET_SLK_REALM_ID != 0 {
            info.realm_id = metadata.realm_id().clone();
        }

        if flags & RSI_ISLET_SLK_SVN != 0 {
            info.svn = svn;
        }
    }

    // We allow realms not having a metadata block assigned
    // to derive sealing keys. In that case, the RIM of the realm
    // is always used as the key material.
    if flags & RSI_ISLET_SLK_RIM != 0 || rd.metadata().is_none() {
        let mut rim = Measurement::empty();
        crate::rsi::measurement::read(&rd, MEASUREMENTS_SLOT_RIM, &mut rim)?;
        info.rim.copy_from_slice(rim.as_slice());
    }

    info.dump();
    info!(
        "ikm type: {}",
        if flags & RSI_ISLET_USE_VHUK_M != 0 {
            "VHUK_M"
        } else {
            "VHUK_A"
        }
    );
    info.derive_sealing_key(flags & RSI_ISLET_USE_VHUK_M != 0, buf);

    // Clear the input key material
    info.zeroize();

    Ok(())
}
