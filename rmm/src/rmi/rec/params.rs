use super::mpidr;
use crate::const_assert_eq;
use crate::granule::{GranuleState, GRANULE_SIZE};
use crate::measurement::Hashable;
use crate::rmi;
use crate::rmi::error::Error;
use crate::{get_granule, get_granule_if};

use autopadding::*;

pub const NR_AUX: usize = 16;
pub const NR_GPRS: usize = 8;

pad_struct_and_impl_default!(
pub struct Params {
    0x0    pub flags: u64,
    0x100  pub mpidr: u64,
    0x200  pub pc: u64,
    0x300  pub gprs: [u64; NR_GPRS],
    0x800  pub num_aux: u64,
    0x808  pub aux: [u64; NR_AUX],
    0x1000 => @END,
}
);
const_assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);

impl Params {
    pub fn verify_compliance(&self, rec: usize, rd: usize, params_ptr: usize) -> Result<(), Error> {
        // Currently, we use rmi::MAX_REC_AUX_GRANULES for RecAuxCount(rd)
        if !mpidr::validate(self.mpidr)
            || self.num_aux as usize == 0
            || self.num_aux as usize > rmi::MAX_REC_AUX_GRANULES
        {
            return Err(Error::RmiErrorInput);
        }

        // Verify that all AUX entries in 0..num_aux are valid, unique, and Delegated
        let num_aux = self.num_aux as usize;
        let mut aux = self.aux;
        let aux_slice = &mut aux[..num_aux];
        aux_slice.sort_unstable();
        for idx in 0..num_aux {
            let addr = aux_slice[idx] as usize;
            if addr == 0 || addr == rec || addr == rd || addr == params_ptr {
                return Err(Error::RmiErrorInput);
            }

            // Check for duplicates
            if idx != 0 && aux_slice[idx - 1] == aux_slice[idx] {
                return Err(Error::RmiErrorInput);
            }

            let _aux_granule = get_granule_if!(addr, GranuleState::Delegated)?;
        }

        Ok(())
    }
}

impl core::fmt::Debug for Params {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Params")
            .field("flags", &format_args!("{:#X}", &self.flags))
            .field("mpidr", &format_args!("{:#X}", &self.mpidr))
            .field("pc", &format_args!("{:#X}", &self.pc))
            .field("gprs", &format_args!("{:#X?}", self.gprs))
            .field("num_aux", &self.num_aux)
            .field("aux", &self.aux)
            .finish()
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

impl Hashable for Params {
    fn hash(
        &self,
        hasher: &crate::measurement::Hasher,
        out: &mut [u8],
    ) -> Result<(), crate::measurement::MeasurementError> {
        hasher.hash_fields_into(out, |h| {
            h.hash_u64(self.flags);
            h.hash(self._padflags);
            h.hash_u64(0); // mpidr not used
            h.hash(self._padmpidr);
            h.hash_u64(self.pc);
            h.hash(self._padpc);
            h.hash_u64_array(self.gprs.as_slice());
            h.hash(self._padgprs);
            h.hash_u64(0); // num_aux not used
            h.hash_u64_array([0u64; 16].as_slice()); // aux is not used
            h.hash(self._padaux);
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{alloc_granule, IDX_RD, IDX_REC1, IDX_REC1_AUX, IDX_REC1_PARAMS};

    #[test]
    fn test_verify_compliance_aux_validation() {
        let rd = alloc_granule(IDX_RD);
        let rec = alloc_granule(IDX_REC1);
        let params_ptr = alloc_granule(IDX_REC1_PARAMS);
        let aux0 = alloc_granule(IDX_REC1_AUX);
        let aux1 = alloc_granule(IDX_REC1_AUX + 1);

        // Zero AUX address inside 0..num_aux must be rejected
        let mut params = Params::default();
        params.num_aux = 2;
        params.aux[0] = aux0 as u64;
        params.aux[1] = 0;
        assert!(matches!(
            params.verify_compliance(rec, rd, params_ptr),
            Err(Error::RmiErrorInput)
        ));

        // Aliasing with rec, rd, or params_ptr must be rejected even when trailing slots are 0
        for aliased in [rec, rd, params_ptr] {
            let mut params = Params::default();
            params.num_aux = 1;
            params.aux[0] = aliased as u64;
            assert!(matches!(
                params.verify_compliance(rec, rd, params_ptr),
                Err(Error::RmiErrorInput)
            ));
        }

        // Duplicate AUX addresses must be rejected even when trailing slots are 0
        let mut params = Params::default();
        params.num_aux = 2;
        params.aux[0] = aux0 as u64;
        params.aux[1] = aux0 as u64;
        assert!(matches!(
            params.verify_compliance(rec, rd, params_ptr),
            Err(Error::RmiErrorInput)
        ));

        // Undelegated AUX granules must be rejected
        let mut params = Params::default();
        params.num_aux = 2;
        params.aux[0] = aux0 as u64;
        params.aux[1] = aux1 as u64;
        assert!(matches!(
            params.verify_compliance(rec, rd, params_ptr),
            Err(Error::RmiErrorInput)
        ));
    }
}
