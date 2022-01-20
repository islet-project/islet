#!/bin/bash

ROOT=$(git rev-parse --show-toplevel)

source ${ROOT}/scripts/env.sh

(
	cd ${FASTMODEL}/Base_RevC_AEMvA_pkg/models/Linux64_GCC-6.4/

	 ./FVP_Base_RevC-2xAEMvA -C bp.flashloader0.fname=${ROOT}/out/fip.bin -C bp.secureflashloader.fname=${ROOT}/out/bl1.bin -C bp.refcounter.non_arch_start_at_default=1 -C bp.refcounter.use_real_time=0 -C bp.ve_sysregs.exit_on_shutdown=1 -C cache_state_modelled=1 -C cluster0.NUM_CORES=4 -C cluster0.PA_SIZE=48 -C cluster0.ecv_support_level=2 -C cluster0.gicv3.cpuintf-mmap-access-level=2 -C cluster0.gicv3.without-DS-support=1 -C cluster0.gicv4.mask-virtual-interrupt=1 -C cluster0.has_arm_v8-6=1 -C cluster0.has_branch_target_exception=1 -C cluster0.has_rme=1 -C cluster0.has_rndr=1 -C cluster0.has_amu=1 -C cluster0.has_v8_7_pmu_extension=2 -C cluster0.max_32bit_el=-1 -C cluster0.restriction_on_speculative_execution=2 -C cluster0.restriction_on_speculative_execution_aarch32=2 -C cluster1.NUM_CORES=4 -C cluster1.PA_SIZE=48 -C cluster1.ecv_support_level=2 -C cluster1.gicv3.cpuintf-mmap-access-level=2 -C cluster1.gicv3.without-DS-support=1 -C cluster1.gicv4.mask-virtual-interrupt=1 -C cluster1.has_arm_v8-6=1 -C cluster1.has_branch_target_exception=1 -C cluster1.has_rme=1 -C cluster1.has_rndr=1 -C cluster1.has_amu=1 -C cluster1.has_v8_7_pmu_extension=2 -C cluster1.max_32bit_el=-1 -C cluster1.restriction_on_speculative_execution=2 -C cluster1.restriction_on_speculative_execution_aarch32=2 -C pci.pci_smmuv3.mmu.SMMU_AIDR=2 -C pci.pci_smmuv3.mmu.SMMU_IDR0=0x0046123B -C pci.pci_smmuv3.mmu.SMMU_IDR1=0x00600002 -C pci.pci_smmuv3.mmu.SMMU_IDR3=0x1714 -C pci.pci_smmuv3.mmu.SMMU_IDR5=0xFFFF0475 -C pci.pci_smmuv3.mmu.SMMU_S_IDR1=0xA0000002 -C pci.pci_smmuv3.mmu.SMMU_S_IDR2=0 -C pci.pci_smmuv3.mmu.SMMU_S_IDR3=0 -C bp.pl011_uart0.out_file=uart0.log -C bp.pl011_uart1.out_file=uart1.log -C bp.pl011_uart2.out_file=uart2.log -C pctl.startup=0.0.0.0 -Q 1000 "$@"
)
