/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#ifndef __ASM_RMI_CMDS_H
#define __ASM_RMI_CMDS_H

#include <linux/arm-smccc.h>

#include <asm/rmi_smc.h>

struct rtt_entry {
	unsigned long walk_level;
	unsigned long desc;
	int state;
	bool ripas;
};

static inline int rmi_data_create(unsigned long data, unsigned long rd,
				  unsigned long map_addr, unsigned long src,
				  unsigned long flags)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_DATA_CREATE, data, rd, map_addr, src,
			     flags, &res);

	return res.a0;
}

static inline int rmi_data_create_unknown(unsigned long data,
					  unsigned long rd,
					  unsigned long map_addr)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_DATA_CREATE_UNKNOWN, data, rd, map_addr,
			     &res);

	return res.a0;
}

static inline int rmi_data_destroy(unsigned long rd, unsigned long map_addr)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_DATA_DESTROY, rd, map_addr, &res);

	return res.a0;
}

static inline int rmi_features(unsigned long index, unsigned long *out)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_FEATURES, index, &res);

	*out = res.a1;
	return res.a0;
}

static inline int rmi_granule_delegate(unsigned long phys)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_GRANULE_DELEGATE, phys, &res);

	return res.a0;
}

static inline int rmi_granule_undelegate(unsigned long phys)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_GRANULE_UNDELEGATE, phys, &res);

	return res.a0;
}

static inline int rmi_psci_complete(unsigned long calling_rec,
				    unsigned long target_rec)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_PSCI_COMPLETE, calling_rec, target_rec,
			     &res);

	return res.a0;
}

static inline int rmi_realm_activate(unsigned long rd)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_REALM_ACTIVATE, rd, &res);

	return res.a0;
}

static inline int rmi_realm_create(unsigned long rd, unsigned long params_ptr)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_REALM_CREATE, rd, params_ptr, &res);

	return res.a0;
}

static inline int rmi_realm_destroy(unsigned long rd)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_REALM_DESTROY, rd, &res);

	return res.a0;
}

static inline int rmi_rec_aux_count(unsigned long rd, unsigned long *aux_count)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_REC_AUX_COUNT, rd, &res);

	*aux_count = res.a1;
	return res.a0;
}

static inline int rmi_rec_create(unsigned long rec, unsigned long rd,
				 unsigned long params_ptr)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_REC_CREATE, rec, rd, params_ptr, &res);

	return res.a0;
}

static inline int rmi_rec_destroy(unsigned long rec)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_REC_DESTROY, rec, &res);

	return res.a0;
}

static inline int rmi_rec_enter(unsigned long rec, unsigned long run_ptr)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_REC_ENTER, rec, run_ptr, &res);

	return res.a0;
}

static inline int rmi_rtt_create(unsigned long rtt, unsigned long rd,
				 unsigned long map_addr, unsigned long level)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_RTT_CREATE, rtt, rd, map_addr, level,
			     &res);

	return res.a0;
}

static inline int rmi_rtt_destroy(unsigned long rtt, unsigned long rd,
				  unsigned long map_addr, unsigned long level)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_RTT_DESTROY, rtt, rd, map_addr, level,
			     &res);

	return res.a0;
}

static inline int rmi_rtt_fold(unsigned long rtt, unsigned long rd,
			       unsigned long map_addr, unsigned long level)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_RTT_FOLD, rtt, rd, map_addr, level, &res);

	return res.a0;
}

static inline int rmi_rtt_init_ripas(unsigned long rd, unsigned long map_addr,
				     unsigned long level)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_RTT_INIT_RIPAS, rd, map_addr, level, &res);

	return res.a0;
}

static inline int rmi_rtt_map_unprotected(unsigned long rd,
					  unsigned long map_addr,
					  unsigned long level,
					  unsigned long desc)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_RTT_MAP_UNPROTECTED, rd, map_addr, level,
			     desc, &res);

	return res.a0;
}

static inline int rmi_rtt_read_entry(unsigned long rd, unsigned long map_addr,
				     unsigned long level, struct rtt_entry *rtt)
{
	struct arm_smccc_1_2_regs regs = {
		SMC_RMI_RTT_READ_ENTRY,
		rd, map_addr, level
	};

	arm_smccc_1_2_smc(&regs, &regs);

	rtt->walk_level = regs.a1;
	rtt->state = regs.a2 & 0xFF;
	rtt->desc = regs.a3;
	rtt->ripas = regs.a4 & 1;

	return regs.a0;
}

static inline int rmi_rtt_set_ripas(unsigned long rd, unsigned long rec,
				    unsigned long map_addr, unsigned long level,
				    unsigned long ripas)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_RTT_SET_RIPAS, rd, rec, map_addr, level,
			     ripas, &res);

	return res.a0;
}

static inline int rmi_rtt_unmap_unprotected(unsigned long rd,
					    unsigned long map_addr,
					    unsigned long level)
{
	struct arm_smccc_res res;

	arm_smccc_1_1_invoke(SMC_RMI_RTT_UNMAP_UNPROTECTED, rd, map_addr,
			     level, &res);

	return res.a0;
}

static inline phys_addr_t rmi_rtt_get_phys(struct rtt_entry *rtt)
{
	return rtt->desc & GENMASK(47, 12);
}

#endif
