use crate::alloc::string::ToString;
use crate::const_assert_eq;
use crate::granule::{GRANULE_SHIFT, GRANULE_SIZE};
use crate::host::Accessor as HostAccessor;
use crate::measurement::Hashable;
use crate::rmi::features;
use crate::rmi::rtt::{RTT_PAGE_LEVEL, S2TTE_STRIDE};
use crate::rmi::{HASH_ALGO_SHA256, HASH_ALGO_SHA512};
use core::ffi::CStr;

const PADDING: [usize; 5] = [248, 767, 960, 6, 2020];

#[repr(C)]
pub struct Params {
    pub features_0: u64,
    padding0: [u8; PADDING[0]],
    pub hash_algo: u8,
    padding1: [u8; PADDING[1]],
    pub rpv: [u8; 64],  // no_shared_region if rpv[0] = 0x1
    padding2: [u8; PADDING[2]],
    pub vmid: u16,
    padding3: [u8; PADDING[3]],
    pub rtt_base: u64,
    pub rtt_level_start: i64,
    pub rtt_num_start: u32,
    padding4: [u8; PADDING[4]],  // expected_measurement for mutual attestation, padding4[0..31] (32-bytes)
}

const_assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);

impl Default for Params {
    fn default() -> Self {
        Self {
            features_0: 0,
            padding0: [0; PADDING[0]],
            hash_algo: 0,
            padding1: [0; PADDING[1]],
            rpv: [0; 64],
            padding2: [0; PADDING[2]],
            vmid: 0,
            padding3: [0; PADDING[3]],
            rtt_base: 0,
            rtt_level_start: 0,
            rtt_num_start: 0,
            padding4: [0; PADDING[4]],
        }
    }
}

impl core::fmt::Debug for Params {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Params")
            .field("features_0", &format_args!("{:#X}", &self.features_0))
            .field("hash_algo", &self.hash_algo)
            .field("rpv", &self.rpv)
            .field("vmid", &self.vmid)
            .field("rtt_base", &format_args!("{:#X}", &self.rtt_base))
            .field("rtt_level_start", &self.rtt_level_start)
            .field("rtt_num_start", &self.rtt_num_start)
            .finish()
    }
}

impl Hashable for Params {
    fn hash(
        &self,
        hasher: &crate::measurement::Hasher,
        out: &mut [u8],
    ) -> Result<(), crate::measurement::MeasurementError> {
        let zero_padding4: [u8; PADDING[4]] = [0; PADDING[4]];

        hasher.hash_fields_into(out, |alg| {
            alg.hash_u64(0); // features aren't used
            alg.hash(self.padding0);
            alg.hash_u8(self.hash_algo);
            alg.hash(self.padding1);
            alg.hash(self.rpv);
            alg.hash(self.padding2);
            alg.hash_u16(0); // vmid is not used
            alg.hash(self.padding3);
            alg.hash_u64(0); // rtt_base is not used
            alg.hash_u64(0); // rtt_level_start is not used
            alg.hash_u32(0); // rtt_num_start is not used
            alg.hash(zero_padding4);  // do not hash it as expected_measurement is in it.
        })
    }
}

impl HostAccessor for Params {
    fn validate(&self) -> bool {
        trace!("{:?}", self);
        if !features::validate(self.features_0 as usize) {
            return false;
        }

        // Check misconfigurations between IPA size and SL
        let ipa_bits = self.ipa_bits();
        let rtt_slvl = self.rtt_level_start as usize;

        let level = RTT_PAGE_LEVEL - rtt_slvl;
        let min_ipa_bits = level * S2TTE_STRIDE + GRANULE_SHIFT + 1;
        let max_ipa_bits = min_ipa_bits + (S2TTE_STRIDE - 1) + 4;

        if (ipa_bits < min_ipa_bits) || (ipa_bits > max_ipa_bits) {
            return false;
        }

        match self.hash_algo {
            HASH_ALGO_SHA256 | HASH_ALGO_SHA512 => true,
            _ => false,
        }
    }
}

impl Params {
    pub fn ipa_bits(&self) -> usize {
        features::ipa_bits(self.features_0 as usize)
    }

    pub fn no_shared_region(&self) -> bool {
        let no_shared_region_str = "no_shared_region".to_string();
        // [TODO] remove unwrap
        let rpv_str = match CStr::from_bytes_until_nul(&self.rpv) {
            Ok(v) => v.to_str().unwrap().to_string(),
            Err(_) => return false,
        };

        info!("[JB] rpv_str: {}, str: {}", rpv_str, no_shared_region_str);
        if rpv_str == no_shared_region_str {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::offset_of;

    #[test]
    fn spec_params() {
        assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);

        assert_eq!(offset_of!(Params, features_0), 0x0);
        assert_eq!(offset_of!(Params, hash_algo), 0x100);
        assert_eq!(offset_of!(Params, rpv), 0x400);
        assert_eq!(offset_of!(Params, vmid), 0x800);
        assert_eq!(offset_of!(Params, rtt_base), 0x808);
        assert_eq!(offset_of!(Params, rtt_level_start), 0x810);
        assert_eq!(offset_of!(Params, rtt_num_start), 0x818);
    }
}
