use crate::const_assert_eq;
use crate::granule::{GRANULE_SHIFT, GRANULE_SIZE};
use crate::measurement::Hashable;
use crate::pmu;
use crate::realm::mm::rtt::{RTT_PAGE_LEVEL, RTT_STRIDE};
use crate::realm::mm::stage2_translation::is_root_rtt_supported;
use crate::rmi::error::Error;
use crate::rmi::features;
use crate::rmi::{HASH_ALGO_SHA256, HASH_ALGO_SHA512};
use crate::simd;

use armv9a::{define_bitfield, define_bits, define_mask};
use autopadding::*;

define_bits!(
    RmiRealmFlags,
    Lpa2[0 - 0],
    Sve[1 - 1],
    Pmu[2 - 2],
    Reserved[63 - 3]
);

pad_struct_and_impl_default!(
pub struct Params {
    0x0    pub flags: u64,
    0x8    pub s2sz: u8,
    0x10   pub sve_vl: u8,
    0x18   pub num_bps: u8,
    0x20   pub num_wps: u8,
    0x28   pub pmu_num_ctrs: u8,
    0x30   pub hash_algo: u8,
    0x400  pub rpv: [u8; 64],
    0x800  pub vmid: u16,
    0x808  pub rtt_base: u64,
    0x810  pub rtt_level_start: i64,
    0x818  pub rtt_num_start: u32,
    0x1000 => @END,
}
);

const_assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);
const SUPPORTED: u64 = 1;

impl core::fmt::Debug for Params {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Params")
            .field(
                "flags",
                &format_args!(
                    "lpa2: {:?} sve: {:?} pmu: {:?}",
                    RmiRealmFlags::new(self.flags).get_masked_value(RmiRealmFlags::Lpa2),
                    RmiRealmFlags::new(self.flags).get_masked_value(RmiRealmFlags::Sve),
                    RmiRealmFlags::new(self.flags).get_masked_value(RmiRealmFlags::Pmu)
                ),
            )
            .field("s2sz", &self.s2sz)
            .field("sve_vl", &self.sve_vl)
            .field("num_bps", &self.num_bps)
            .field("num_wps", &self.num_wps)
            .field("pmu_num_ctrs", &self.pmu_num_ctrs)
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
            alg.hash_u64(self.flags);
            alg.hash(self._padflags);
            alg.hash_u8(self.s2sz);
            alg.hash(self._pads2sz);
            alg.hash_u8(self.sve_vl);
            alg.hash(self._padsve_vl);
            alg.hash_u8(self.num_bps);
            alg.hash(self._padnum_bps);
            alg.hash_u8(self.num_wps);
            alg.hash(self._padnum_wps);
            alg.hash_u8(self.pmu_num_ctrs);
            alg.hash(self._padpmu_num_ctrs);
            alg.hash_u8(self.hash_algo);
            alg.hash(self._padhash_algo);
            alg.hash([0u8; 64]); // rpv is not used
            alg.hash(self._padrpv);
            alg.hash_u16(0); // vmid is not used
            alg.hash(self._padvmid);
            alg.hash_u64(0); // rtt_base is not used
            alg.hash(self._padrtt_base);
            alg.hash_u64(0); // rtt_level_start is not used
            alg.hash(self._padrtt_level_start);
            alg.hash_u32(0); // rtt_num_start is not used
            alg.hash(self._padrtt_num_start);
        })
    }
}

impl Params {
    pub fn ipa_bits(&self) -> usize {
        self.s2sz as usize
    }

    pub fn sve_en(&self) -> bool {
        let flags = RmiRealmFlags::new(self.flags);
        flags.get_masked_value(RmiRealmFlags::Sve) == SUPPORTED
    }

    pub fn pmu_en(&self) -> bool {
        let flags = RmiRealmFlags::new(self.flags);
        flags.get_masked_value(RmiRealmFlags::Pmu) == SUPPORTED
    }

    pub fn verify_compliance(&self, rd: usize) -> Result<(), Error> {
        trace!("{:?}", self);
        if !(self.rtt_base as usize).is_multiple_of(GRANULE_SIZE) {
            return Err(Error::RmiErrorInput);
        }

        if !features::validate(self.s2sz as usize) {
            return Err(Error::RmiErrorInput);
        }

        // Check misconfigurations between IPA size and SL
        let ipa_bits = self.ipa_bits();
        let rtt_slvl = self.rtt_level_start as usize;

        let level = RTT_PAGE_LEVEL
            .checked_sub(rtt_slvl)
            .ok_or(Error::RmiErrorInput)?;
        let min_ipa_bits = level * RTT_STRIDE + GRANULE_SHIFT + 1;
        let max_ipa_bits = min_ipa_bits + (RTT_STRIDE - 1) + 4;
        let sl_ipa_bits = (level * RTT_STRIDE) + GRANULE_SHIFT + RTT_STRIDE;

        if (ipa_bits < min_ipa_bits) || (ipa_bits > max_ipa_bits) {
            return Err(Error::RmiErrorInput);
        }

        let s2_num_root_rtts = {
            if sl_ipa_bits >= ipa_bits {
                1
            } else {
                1 << (ipa_bits - sl_ipa_bits)
            }
        };
        if s2_num_root_rtts != self.rtt_num_start {
            return Err(Error::RmiErrorInput);
        }

        // Reject the configurations that the stage 2 translation cannot build,
        // instead of failing later in REALM_CREATE.
        // (`rtt_slvl` is known to be in 0..=3 here.)
        if !is_root_rtt_supported(rtt_slvl, self.rtt_num_start as usize) {
            return Err(Error::RmiErrorInput);
        }

        // rtt_align: the concatenated root RTTs must be aligned to their total
        // size. Otherwise the hardware ignores the low bits of VTTBR_EL2.BADDR
        // and walks a table that is not the one the RMM has built.
        let root_size = (self.rtt_num_start as usize) * GRANULE_SIZE;
        let rtt_start = self.rtt_base as usize;
        if !rtt_start.is_multiple_of(root_size) {
            return Err(Error::RmiErrorInput);
        }

        // The RD must not be one of the root RTT granules.
        if rd >= rtt_start && rd < rtt_start + root_size {
            return Err(Error::RmiErrorInput);
        }

        // TODO: We don't support pmu, lpa2
        let flags = RmiRealmFlags::new(self.flags);
        if flags.get_masked_value(RmiRealmFlags::Lpa2) != features::LPA2_VALUE {
            return Err(Error::RmiErrorInput);
        }
        if !simd::validate(self.sve_en(), self.sve_vl as u64) {
            return Err(Error::RmiErrorInput);
        }
        if self.pmu_en()
            && (!pmu::pmu_present()
                || self.pmu_num_ctrs > pmu::pmu_num_ctrs() as u8
                || (self.pmu_num_ctrs == 0 && !pmu::hpmn0_present()))
        {
            return Err(Error::RmiErrorInput);
        }

        match self.hash_algo {
            HASH_ALGO_SHA256 | HASH_ALGO_SHA512 => Ok(()),
            _ => Err(Error::RmiErrorInput),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // aligned to 1MB, which is larger than the biggest (16 granules) root RTT
    const RTT_BASE: u64 = 0x10_0000;
    const RD: usize = 0x1000;

    fn params(s2sz: u8, level: i64, num: u32, base: u64) -> Params {
        let mut p = Params::default();
        p.s2sz = s2sz;
        p.rtt_level_start = level;
        p.rtt_num_start = num;
        p.rtt_base = base;
        p
    }

    #[test]
    fn accepts_concatenated_root_rtt() {
        // (s2sz, rtt_level_start, rtt_num_start)
        let cases = [
            (40, 0, 1),
            (40, 1, 2),
            (41, 1, 4), // L1N4: used to hang the PE in REALM_CREATE
            (42, 1, 8),
            (43, 1, 16), // L1N16: used to hang the PE in REALM_CREATE
            (32, 2, 4),
            (34, 2, 16),
        ];
        for (s2sz, level, num) in cases {
            let p = params(s2sz, level, num, RTT_BASE);
            assert!(
                p.verify_compliance(RD).is_ok(),
                "s2sz={} level={} num={}",
                s2sz,
                level,
                num
            );
        }
    }

    #[test]
    fn rejects_root_rtt_not_aligned_to_its_size() {
        for (s2sz, level, num) in [(40, 1, 2), (41, 1, 4), (42, 1, 8), (43, 1, 16)] {
            // granule aligned, but not aligned to num * GRANULE_SIZE
            let base = RTT_BASE + GRANULE_SIZE as u64;
            let p = params(s2sz, level, num, base);
            assert!(
                p.verify_compliance(RD).is_err(),
                "s2sz={} level={} num={}",
                s2sz,
                level,
                num
            );
        }
    }

    #[test]
    fn rejects_rd_inside_root_rtt_range() {
        let root_size = 4 * GRANULE_SIZE;
        let p = params(41, 1, 4, RTT_BASE);
        let base = RTT_BASE as usize;

        for rd in [base, base + GRANULE_SIZE, base + root_size - GRANULE_SIZE] {
            assert!(p.verify_compliance(rd).is_err(), "rd={:#x}", rd);
        }
        for rd in [base - GRANULE_SIZE, base + root_size] {
            assert!(p.verify_compliance(rd).is_ok(), "rd={:#x}", rd);
        }
    }

    #[test]
    fn rejects_inconsistent_root_rtt_count() {
        // s2sz=41 at level 1 needs 4 root RTTs
        for num in [0, 1, 2, 3, 5, 8, 16] {
            let p = params(41, 1, num, RTT_BASE);
            assert!(p.verify_compliance(RD).is_err(), "num={}", num);
        }
    }

    // Whatever verify_compliance() accepts must be buildable by
    // Stage2Translation::new(), otherwise REALM_CREATE panics after the
    // validation and the PE hangs in EL2.
    #[test]
    fn accepted_params_are_supported_by_stage2_translation() {
        let mut accepted = 0;
        for level in -1i64..=4 {
            for s2sz in 28u8..=52 {
                for num in [0u32, 1, 2, 4, 8, 16] {
                    let p = params(s2sz, level, num, RTT_BASE);
                    if p.verify_compliance(RD).is_ok() {
                        accepted += 1;
                        assert!(
                            is_root_rtt_supported(level as usize, num as usize),
                            "accepted but unsupported: s2sz={} level={} num={}",
                            s2sz,
                            level,
                            num
                        );
                    }
                }
            }
        }
        assert!(accepted > 0);
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
