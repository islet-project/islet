/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#ifndef __ASM_RME_SMC_H
#define __ASM_RME_SMC_H

#ifdef CONFIG_RME_USE_PROTOTYPE_HACKS

// Allow the prototype RMI version
#define PROTOTYPE_RMI_ABI_MAJOR_VERSION  56

#endif /* CONFIG_RME_USE_PROTOTYPE_HACKS */

#include <linux/arm-smccc.h>

#define SMC_RxI_CALL(func)				\
	ARM_SMCCC_CALL_VAL(ARM_SMCCC_FAST_CALL,		\
			   ARM_SMCCC_SMC_64,		\
			   ARM_SMCCC_OWNER_STANDARD,	\
			   (func))

/* FID numbers from alp10 specification */

#define SMC_RMI_DATA_CREATE		SMC_RxI_CALL(0x0153)
#define SMC_RMI_DATA_CREATE_UNKNOWN	SMC_RxI_CALL(0x0154)
#define SMC_RMI_DATA_DESTROY		SMC_RxI_CALL(0x0155)
#define SMC_RMI_FEATURES		SMC_RxI_CALL(0x0165)
#define SMC_RMI_GRANULE_DELEGATE	SMC_RxI_CALL(0x0151)
#define SMC_RMI_GRANULE_UNDELEGATE	SMC_RxI_CALL(0x0152)
#define SMC_RMI_PSCI_COMPLETE		SMC_RxI_CALL(0x0164)
#define SMC_RMI_REALM_ACTIVATE		SMC_RxI_CALL(0x0157)
#define SMC_RMI_REALM_CREATE		SMC_RxI_CALL(0x0158)
#define SMC_RMI_REALM_DESTROY		SMC_RxI_CALL(0x0159)
#define SMC_RMI_REC_AUX_COUNT		SMC_RxI_CALL(0x0167)
#define SMC_RMI_REC_CREATE		SMC_RxI_CALL(0x015a)
#define SMC_RMI_REC_DESTROY		SMC_RxI_CALL(0x015b)
#define SMC_RMI_REC_ENTER		SMC_RxI_CALL(0x015c)
#define SMC_RMI_RTT_CREATE		SMC_RxI_CALL(0x015d)
#define SMC_RMI_RTT_DESTROY		SMC_RxI_CALL(0x015e)
#define SMC_RMI_RTT_FOLD		SMC_RxI_CALL(0x0166)
#define SMC_RMI_RTT_INIT_RIPAS		SMC_RxI_CALL(0x0168)
#define SMC_RMI_RTT_MAP_UNPROTECTED	SMC_RxI_CALL(0x015f)
#define SMC_RMI_RTT_READ_ENTRY		SMC_RxI_CALL(0x0161)
#define SMC_RMI_RTT_SET_RIPAS		SMC_RxI_CALL(0x0169)
#define SMC_RMI_RTT_UNMAP_UNPROTECTED	SMC_RxI_CALL(0x0162)
#define SMC_RMI_VERSION			SMC_RxI_CALL(0x0150)

#define RMI_ABI_MAJOR_VERSION	1
#define RMI_ABI_MINOR_VERSION	0

#define RMI_UNASSIGNED			0
#define RMI_DESTROYED			1
#define RMI_ASSIGNED			2
#define RMI_TABLE			3
#define RMI_VALID_NS			4

#define RMI_ABI_VERSION_GET_MAJOR(version) ((version) >> 16)
#define RMI_ABI_VERSION_GET_MINOR(version) ((version) & 0xFFFF)

#define RMI_RETURN_STATUS(ret)		((ret) & 0xFF)
#define RMI_RETURN_INDEX(ret)		(((ret) >> 8) & 0xFF)

#define RMI_SUCCESS		0
#define RMI_ERROR_INPUT		1
#define RMI_ERROR_REALM		2
#define RMI_ERROR_REC		3
#define RMI_ERROR_RTT		4
#define RMI_ERROR_IN_USE	5

#define RMI_EMPTY		0
#define RMI_RAM			1

#define RMI_NO_MEASURE_CONTENT	0
#define RMI_MEASURE_CONTENT	1

#define RMI_FEATURE_REGISTER_0_S2SZ		GENMASK(7, 0)
#define RMI_FEATURE_REGISTER_0_LPA2		BIT(8)
#define RMI_FEATURE_REGISTER_0_SVE_EN		BIT(9)
#define RMI_FEATURE_REGISTER_0_SVE_VL		GENMASK(13, 10)
#define RMI_FEATURE_REGISTER_0_NUM_BPS		GENMASK(17, 14)
#define RMI_FEATURE_REGISTER_0_NUM_WPS		GENMASK(21, 18)
#define RMI_FEATURE_REGISTER_0_PMU_EN		BIT(22)
#define RMI_FEATURE_REGISTER_0_PMU_NUM_CTRS	GENMASK(27, 23)
#define RMI_FEATURE_REGISTER_0_HASH_SHA_256	BIT(28)
#define RMI_FEATURE_REGISTER_0_HASH_SHA_512	BIT(29)

struct realm_params {
	union {
		u64 features_0;
		u8 padding_1[0x100];
	};
	union {
		u8 measurement_algo;
		u8 padding_2[0x300];
	};
	union {
		u8 rpv[64];
		u8 padding_3[0x400];
	};
	union {
		struct {
			u16 vmid;
			u8 padding_4[6];
			u64 rtt_base;
			u64 rtt_level_start;
			u32 rtt_num_start;
		};
		u8 padding_5[0x800];
	};
};

/*
 * The number of GPRs (starting from X0) that are
 * configured by the host when a REC is created.
 */
#define REC_CREATE_NR_GPRS		8

#define REC_PARAMS_FLAG_RUNNABLE	BIT_ULL(0)

#define REC_PARAMS_AUX_GRANULES		16

struct rec_params {
	union {
		u64 flags;
		u8 padding1[0x100];
	};
	union {
		u64 mpidr;
		u8 padding2[0x100];
	};
	union {
		u64 pc;
		u8 padding3[0x100];
	};
	union {
		u64 gprs[REC_CREATE_NR_GPRS];
		u8 padding4[0x500];
	};
	u64 num_rec_aux;
	u64 aux[REC_PARAMS_AUX_GRANULES];
};

#define RMI_EMULATED_MMIO		BIT(0)
#define RMI_INJECT_SEA			BIT(1)
#define RMI_TRAP_WFI			BIT(2)
#define RMI_TRAP_WFE			BIT(3)

#define REC_RUN_GPRS			31
#define REC_GIC_NUM_LRS			16

struct rec_entry {
	union { /* 0x000 */
		u64 flags;
		u8 padding0[0x200];
	};
	union { /* 0x200 */
		u64 gprs[REC_RUN_GPRS];
		u8 padding2[0x100];
	};
	union { /* 0x300 */
		struct {
			u64 gicv3_hcr;
			u64 gicv3_lrs[REC_GIC_NUM_LRS];
		};
		u8 padding3[0x100];
	};
	u8 padding4[0x400];
};

struct rec_exit {
	union { /* 0x000 */
		u8 exit_reason;
		u8 padding0[0x100];
	};
	union { /* 0x100 */
		struct {
			u64 esr;
			u64 far;
			u64 hpfar;
		};
		u8 padding1[0x100];
	};
	union { /* 0x200 */
		u64 gprs[REC_RUN_GPRS];
		u8 padding2[0x100];
	};
	union { /* 0x300 */
		struct {
			u64 gicv3_hcr;
			u64 gicv3_lrs[REC_GIC_NUM_LRS];
			u64 gicv3_misr;
			u64 gicv3_vmcr;
		};
		u8 padding3[0x100];
	};
	union { /* 0x400 */
		struct {
			u64 cntp_ctl;
			u64 cntp_cval;
			u64 cntv_ctl;
			u64 cntv_cval;
		};
		u8 padding4[0x100];
	};
	union { /* 0x500 */
		struct {
			u64 ripas_base;
			u64 ripas_size;
			u64 ripas_value; /* Only lowest bit */
		};
		u8 padding5[0x100];
	};
	union { /* 0x600 */
		u16 imm;
		u8 padding6[0x100];
	};
	union { /* 0x700 */
		struct {
			u64 pmu_ovf;
			u64 pmu_intr_en;
			u64 pmu_cntr_en;
		};
		u8 padding7[0x100];
	};
};

struct rec_run {
	struct rec_entry entry;
	struct rec_exit exit;
};

#define RMI_EXIT_SYNC			0x00
#define RMI_EXIT_IRQ			0x01
#define RMI_EXIT_FIQ			0x02
#define RMI_EXIT_PSCI			0x03
#define RMI_EXIT_RIPAS_CHANGE		0x04
#define RMI_EXIT_HOST_CALL		0x05
#define RMI_EXIT_SERROR			0x06

#endif
