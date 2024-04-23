use crate::const_assert_eq;
use crate::granule::{GRANULE_SHIFT, GRANULE_SIZE};
use crate::measurement::Hashable;
use crate::rmi::error::Error;
use crate::rmi::features;
use crate::rmi::rtt::{RTT_PAGE_LEVEL, S2TTE_STRIDE};
use crate::rmi::{HASH_ALGO_SHA256, HASH_ALGO_SHA512};

use autopadding::*;

pad_struct_and_impl_default!(
pub struct Params {
    0x0    pub features_0: u64,
    0x100  pub hash_algo: u8,
    0x400  pub rpv: [u8; 64],
    0x800  pub vmid: u16,
    0x808  pub rtt_base: u64,
    0x810  pub rtt_level_start: i64,
    0x818  pub rtt_num_start: u32,
    0x1000 => @END,
}
);

const_assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);

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
        hasher.hash_fields_into(out, |alg| {
            alg.hash_u64(0); // features aren't used
            alg.hash(self._padfeatures_0);
            alg.hash_u8(self.hash_algo);
            alg.hash(self._padhash_algo);
            alg.hash([0u8; 64]); // rpv is not used
            alg.hash(self._padrpv);
            alg.hash_u16(0); // vmid is not used
            alg.hash(self._padvmid);
            alg.hash_u64(0); // rtt_base is not used
            alg.hash_u64(0); // rtt_level_start is not used
            alg.hash_u32(0); // rtt_num_start is not used
            alg.hash(self._padrtt_num_start);
        })
    }
}

impl Params {
    pub fn ipa_bits(&self) -> usize {
        features::ipa_bits(self.features_0 as usize)
    }

    pub fn verify_compliance(&self, rd: usize) -> Result<(), Error> {
        if self.rtt_base as usize == rd {
            return Err(Error::RmiErrorInput);
        }

        if self.rtt_base as usize % GRANULE_SIZE != 0 {
            return Err(Error::RmiErrorInput);
        }

        if !features::validate(self.features_0 as usize) {
            return Err(Error::RmiErrorInput);
        }

        // Check misconfigurations between IPA size and SL
        let ipa_bits = self.ipa_bits();
        let rtt_slvl = self.rtt_level_start as usize;

        let level = RTT_PAGE_LEVEL - rtt_slvl;
        let min_ipa_bits = level * S2TTE_STRIDE + GRANULE_SHIFT + 1;
        let max_ipa_bits = min_ipa_bits + (S2TTE_STRIDE - 1) + 4;

        if (ipa_bits < min_ipa_bits) || (ipa_bits > max_ipa_bits) {
            return Err(Error::RmiErrorInput);
        }

        match self.hash_algo {
            HASH_ALGO_SHA256 | HASH_ALGO_SHA512 => Ok(()),
            _ => Err(Error::RmiErrorInput),
        }
    }
}

impl safe_abstraction::raw_ptr::RawPtr for Params {}

impl safe_abstraction::raw_ptr::SafetyChecked for Params {}

impl safe_abstraction::raw_ptr::SafetyAssured for Params {
    fn is_initialized(&self) -> bool {
        // Given the fact that this memory is initialized by the Host,
        // it's not possible to unequivocally guarantee
        // that the values have been initialized from the perspective of the RMM.
        // However, any values, whether correctly initialized or not, will undergo
        // verification during the Measurement phase.
        // Consequently, this function returns `true`.
        true
    }

    fn verify_ownership(&self) -> bool {
        // This memory has permissions from the Host's perspective,
        // which inherently implies that exclusive ownership cannot be guaranteed by the RMM alone.
        // However, since the RMM only performs read operations and any incorrect values will be
        // verified during the Measurement phase.
        // Consequently, this function returns `true`.
        true
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
