#![allow(unused_imports)]
#![allow(unused_attributes)]

use tock_registers::interfaces::{Readable, Writeable};

#[macro_export]
macro_rules! define_pmu_register {
    ($mod_name:ident, $reg_name:ident, $reg_literal:tt) => {
        pub mod $mod_name {
            use tock_registers::interfaces::{Readable, Writeable};
            pub struct Reg;

            impl Readable for Reg {
                type T = u64;
                type R = ();

                sys_coproc_read_raw!(u64, $reg_literal, "x");
            }

            impl Writeable for Reg {
                type T = u64;
                type R = ();

                sys_coproc_write_raw!(u64, $reg_literal, "x");
            }

            pub const $reg_name: Reg = Reg {};
        }
    };
}

define_pmu_register!(pmccfiltr_el0, PMCCFILTR_EL0, "PMCCFILTR_EL0");
define_pmu_register!(pmccntr_el0, PMCCNTR_EL0, "PMCCNTR_EL0");
define_pmu_register!(pmcntenset_el0, PMCNTENSET_EL0, "PMCNTENSET_EL0");
define_pmu_register!(pmcntenclr_el0, PMCNTENCLR_EL0, "PMCNTENCLR_EL0");
define_pmu_register!(pmintenset_el1, PMINTENSET_EL1, "PMINTENSET_EL1");
define_pmu_register!(pmintenclr_el1, PMINTENCLR_EL1, "PMINTENCLR_EL1");
define_pmu_register!(pmovsset_el0, PMOVSSET_EL0, "PMOVSSET_EL0");
define_pmu_register!(pmovsclr_el0, PMOVSCLR_EL0, "PMOVSCLR_EL0");
define_pmu_register!(pmselr_el0, PMSELR_EL0, "PMSELR_EL0");
define_pmu_register!(pmuserenr_el0, PMUSERENR_EL0, "PMUSERENR_EL0");
define_pmu_register!(pmxevcntr_el0, PMXEVCNTR_EL0, "PMXEVCNTR_EL0");
define_pmu_register!(pmxevtyper_el0, PMXEVTYPER_EL0, "PMXEVTYPER_EL0");
define_pmu_register!(pmevcntr0_el0, PMEVCNTR0_EL0, "PMEVCNTR0_EL0");
define_pmu_register!(pmevcntr1_el0, PMEVCNTR1_EL0, "PMEVCNTR1_EL0");
define_pmu_register!(pmevcntr2_el0, PMEVCNTR2_EL0, "PMEVCNTR2_EL0");
define_pmu_register!(pmevcntr3_el0, PMEVCNTR3_EL0, "PMEVCNTR3_EL0");
define_pmu_register!(pmevcntr4_el0, PMEVCNTR4_EL0, "PMEVCNTR4_EL0");
define_pmu_register!(pmevcntr5_el0, PMEVCNTR5_EL0, "PMEVCNTR5_EL0");
define_pmu_register!(pmevcntr6_el0, PMEVCNTR6_EL0, "PMEVCNTR6_EL0");
define_pmu_register!(pmevcntr7_el0, PMEVCNTR7_EL0, "PMEVCNTR7_EL0");
define_pmu_register!(pmevcntr8_el0, PMEVCNTR8_EL0, "PMEVCNTR8_EL0");
define_pmu_register!(pmevcntr9_el0, PMEVCNTR9_EL0, "PMEVCNTR9_EL0");
define_pmu_register!(pmevcntr10_el0, PMEVCNTR10_EL0, "PMEVCNTR10_EL0");
define_pmu_register!(pmevcntr11_el0, PMEVCNTR11_EL0, "PMEVCNTR11_EL0");
define_pmu_register!(pmevcntr12_el0, PMEVCNTR12_EL0, "PMEVCNTR12_EL0");
define_pmu_register!(pmevcntr13_el0, PMEVCNTR13_EL0, "PMEVCNTR13_EL0");
define_pmu_register!(pmevcntr14_el0, PMEVCNTR14_EL0, "PMEVCNTR14_EL0");
define_pmu_register!(pmevcntr15_el0, PMEVCNTR15_EL0, "PMEVCNTR15_EL0");
define_pmu_register!(pmevcntr16_el0, PMEVCNTR16_EL0, "PMEVCNTR16_EL0");
define_pmu_register!(pmevcntr17_el0, PMEVCNTR17_EL0, "PMEVCNTR17_EL0");
define_pmu_register!(pmevcntr18_el0, PMEVCNTR18_EL0, "PMEVCNTR18_EL0");
define_pmu_register!(pmevcntr19_el0, PMEVCNTR19_EL0, "PMEVCNTR19_EL0");
define_pmu_register!(pmevcntr20_el0, PMEVCNTR20_EL0, "PMEVCNTR20_EL0");
define_pmu_register!(pmevcntr21_el0, PMEVCNTR21_EL0, "PMEVCNTR21_EL0");
define_pmu_register!(pmevcntr22_el0, PMEVCNTR22_EL0, "PMEVCNTR22_EL0");
define_pmu_register!(pmevcntr23_el0, PMEVCNTR23_EL0, "PMEVCNTR23_EL0");
define_pmu_register!(pmevcntr24_el0, PMEVCNTR24_EL0, "PMEVCNTR24_EL0");
define_pmu_register!(pmevcntr25_el0, PMEVCNTR25_EL0, "PMEVCNTR25_EL0");
define_pmu_register!(pmevcntr26_el0, PMEVCNTR26_EL0, "PMEVCNTR26_EL0");
define_pmu_register!(pmevcntr27_el0, PMEVCNTR27_EL0, "PMEVCNTR27_EL0");
define_pmu_register!(pmevcntr28_el0, PMEVCNTR28_EL0, "PMEVCNTR28_EL0");
define_pmu_register!(pmevcntr29_el0, PMEVCNTR29_EL0, "PMEVCNTR29_EL0");
define_pmu_register!(pmevcntr30_el0, PMEVCNTR30_EL0, "PMEVCNTR30_EL0");
define_pmu_register!(pmevtyper0_el0, PMEVTYPER0_EL0, "PMEVTYPER0_EL0");
define_pmu_register!(pmevtyper1_el0, PMEVTYPER1_EL0, "PMEVTYPER1_EL0");
define_pmu_register!(pmevtyper2_el0, PMEVTYPER2_EL0, "PMEVTYPER2_EL0");
define_pmu_register!(pmevtyper3_el0, PMEVTYPER3_EL0, "PMEVTYPER3_EL0");
define_pmu_register!(pmevtyper4_el0, PMEVTYPER4_EL0, "PMEVTYPER4_EL0");
define_pmu_register!(pmevtyper5_el0, PMEVTYPER5_EL0, "PMEVTYPER5_EL0");
define_pmu_register!(pmevtyper6_el0, PMEVTYPER6_EL0, "PMEVTYPER6_EL0");
define_pmu_register!(pmevtyper7_el0, PMEVTYPER7_EL0, "PMEVTYPER7_EL0");
define_pmu_register!(pmevtyper8_el0, PMEVTYPER8_EL0, "PMEVTYPER8_EL0");
define_pmu_register!(pmevtyper9_el0, PMEVTYPER9_EL0, "PMEVTYPER9_EL0");
define_pmu_register!(pmevtyper10_el0, PMEVTYPER10_EL0, "PMEVTYPER10_EL0");
define_pmu_register!(pmevtyper11_el0, PMEVTYPER11_EL0, "PMEVTYPER11_EL0");
define_pmu_register!(pmevtyper12_el0, PMEVTYPER12_EL0, "PMEVTYPER12_EL0");
define_pmu_register!(pmevtyper13_el0, PMEVTYPER13_EL0, "PMEVTYPER13_EL0");
define_pmu_register!(pmevtyper14_el0, PMEVTYPER14_EL0, "PMEVTYPER14_EL0");
define_pmu_register!(pmevtyper15_el0, PMEVTYPER15_EL0, "PMEVTYPER15_EL0");
define_pmu_register!(pmevtyper16_el0, PMEVTYPER16_EL0, "PMEVTYPER16_EL0");
define_pmu_register!(pmevtyper17_el0, PMEVTYPER17_EL0, "PMEVTYPER17_EL0");
define_pmu_register!(pmevtyper18_el0, PMEVTYPER18_EL0, "PMEVTYPER18_EL0");
define_pmu_register!(pmevtyper19_el0, PMEVTYPER19_EL0, "PMEVTYPER19_EL0");
define_pmu_register!(pmevtyper20_el0, PMEVTYPER20_EL0, "PMEVTYPER20_EL0");
define_pmu_register!(pmevtyper21_el0, PMEVTYPER21_EL0, "PMEVTYPER21_EL0");
define_pmu_register!(pmevtyper22_el0, PMEVTYPER22_EL0, "PMEVTYPER22_EL0");
define_pmu_register!(pmevtyper23_el0, PMEVTYPER23_EL0, "PMEVTYPER23_EL0");
define_pmu_register!(pmevtyper24_el0, PMEVTYPER24_EL0, "PMEVTYPER24_EL0");
define_pmu_register!(pmevtyper25_el0, PMEVTYPER25_EL0, "PMEVTYPER25_EL0");
define_pmu_register!(pmevtyper26_el0, PMEVTYPER26_EL0, "PMEVTYPER26_EL0");
define_pmu_register!(pmevtyper27_el0, PMEVTYPER27_EL0, "PMEVTYPER27_EL0");
define_pmu_register!(pmevtyper28_el0, PMEVTYPER28_EL0, "PMEVTYPER28_EL0");
define_pmu_register!(pmevtyper29_el0, PMEVTYPER29_EL0, "PMEVTYPER29_EL0");
define_pmu_register!(pmevtyper30_el0, PMEVTYPER30_EL0, "PMEVTYPER30_EL0");

pub use pmccfiltr_el0::PMCCFILTR_EL0;
pub use pmccntr_el0::PMCCNTR_EL0;
pub use pmcntenclr_el0::PMCNTENCLR_EL0;
pub use pmcntenset_el0::PMCNTENSET_EL0;
pub use pmevcntr0_el0::PMEVCNTR0_EL0;
pub use pmevcntr10_el0::PMEVCNTR10_EL0;
pub use pmevcntr11_el0::PMEVCNTR11_EL0;
pub use pmevcntr12_el0::PMEVCNTR12_EL0;
pub use pmevcntr13_el0::PMEVCNTR13_EL0;
pub use pmevcntr14_el0::PMEVCNTR14_EL0;
pub use pmevcntr15_el0::PMEVCNTR15_EL0;
pub use pmevcntr16_el0::PMEVCNTR16_EL0;
pub use pmevcntr17_el0::PMEVCNTR17_EL0;
pub use pmevcntr18_el0::PMEVCNTR18_EL0;
pub use pmevcntr19_el0::PMEVCNTR19_EL0;
pub use pmevcntr1_el0::PMEVCNTR1_EL0;
pub use pmevcntr20_el0::PMEVCNTR20_EL0;
pub use pmevcntr21_el0::PMEVCNTR21_EL0;
pub use pmevcntr22_el0::PMEVCNTR22_EL0;
pub use pmevcntr23_el0::PMEVCNTR23_EL0;
pub use pmevcntr24_el0::PMEVCNTR24_EL0;
pub use pmevcntr25_el0::PMEVCNTR25_EL0;
pub use pmevcntr26_el0::PMEVCNTR26_EL0;
pub use pmevcntr27_el0::PMEVCNTR27_EL0;
pub use pmevcntr28_el0::PMEVCNTR28_EL0;
pub use pmevcntr29_el0::PMEVCNTR29_EL0;
pub use pmevcntr2_el0::PMEVCNTR2_EL0;
pub use pmevcntr30_el0::PMEVCNTR30_EL0;
pub use pmevcntr3_el0::PMEVCNTR3_EL0;
pub use pmevcntr4_el0::PMEVCNTR4_EL0;
pub use pmevcntr5_el0::PMEVCNTR5_EL0;
pub use pmevcntr6_el0::PMEVCNTR6_EL0;
pub use pmevcntr7_el0::PMEVCNTR7_EL0;
pub use pmevcntr8_el0::PMEVCNTR8_EL0;
pub use pmevcntr9_el0::PMEVCNTR9_EL0;
pub use pmevtyper0_el0::PMEVTYPER0_EL0;
pub use pmevtyper10_el0::PMEVTYPER10_EL0;
pub use pmevtyper11_el0::PMEVTYPER11_EL0;
pub use pmevtyper12_el0::PMEVTYPER12_EL0;
pub use pmevtyper13_el0::PMEVTYPER13_EL0;
pub use pmevtyper14_el0::PMEVTYPER14_EL0;
pub use pmevtyper15_el0::PMEVTYPER15_EL0;
pub use pmevtyper16_el0::PMEVTYPER16_EL0;
pub use pmevtyper17_el0::PMEVTYPER17_EL0;
pub use pmevtyper18_el0::PMEVTYPER18_EL0;
pub use pmevtyper19_el0::PMEVTYPER19_EL0;
pub use pmevtyper1_el0::PMEVTYPER1_EL0;
pub use pmevtyper20_el0::PMEVTYPER20_EL0;
pub use pmevtyper21_el0::PMEVTYPER21_EL0;
pub use pmevtyper22_el0::PMEVTYPER22_EL0;
pub use pmevtyper23_el0::PMEVTYPER23_EL0;
pub use pmevtyper24_el0::PMEVTYPER24_EL0;
pub use pmevtyper25_el0::PMEVTYPER25_EL0;
pub use pmevtyper26_el0::PMEVTYPER26_EL0;
pub use pmevtyper27_el0::PMEVTYPER27_EL0;
pub use pmevtyper28_el0::PMEVTYPER28_EL0;
pub use pmevtyper29_el0::PMEVTYPER29_EL0;
pub use pmevtyper2_el0::PMEVTYPER2_EL0;
pub use pmevtyper30_el0::PMEVTYPER30_EL0;
pub use pmevtyper3_el0::PMEVTYPER3_EL0;
pub use pmevtyper4_el0::PMEVTYPER4_EL0;
pub use pmevtyper5_el0::PMEVTYPER5_EL0;
pub use pmevtyper6_el0::PMEVTYPER6_EL0;
pub use pmevtyper7_el0::PMEVTYPER7_EL0;
pub use pmevtyper8_el0::PMEVTYPER8_EL0;
pub use pmevtyper9_el0::PMEVTYPER9_EL0;
pub use pmintenclr_el1::PMINTENCLR_EL1;
pub use pmintenset_el1::PMINTENSET_EL1;
pub use pmovsclr_el0::PMOVSCLR_EL0;
pub use pmovsset_el0::PMOVSSET_EL0;
pub use pmselr_el0::PMSELR_EL0;
pub use pmuserenr_el0::PMUSERENR_EL0;
pub use pmxevcntr_el0::PMXEVCNTR_EL0;
pub use pmxevtyper_el0::PMXEVTYPER_EL0;
