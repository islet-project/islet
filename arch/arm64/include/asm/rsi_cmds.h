/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#ifndef __ASM_RSI_CMDS_H
#define __ASM_RSI_CMDS_H

#include <linux/arm-smccc.h>

#include <asm/rsi_smc.h>

enum ripas {
	RSI_RIPAS_EMPTY,
	RSI_RIPAS_RAM,
};

static inline unsigned long rsi_get_version(void)
{
	struct arm_smccc_res res;

	arm_smccc_smc(SMC_RSI_ABI_VERSION, 0, 0, 0, 0, 0, 0, 0, &res);

	return res.a0;
}

static inline unsigned long invoke_rsi_fn_smc(unsigned long function_id,
					      unsigned long arg0,
					      unsigned long arg1,
					      unsigned long arg2,
					      unsigned long arg3)
{
	struct arm_smccc_res res;

	arm_smccc_smc(function_id, arg0, arg1, arg2, arg3, 0, 0, 0, &res);
	return res.a0;
}

static inline void invoke_rsi_fn_smc_with_res(unsigned long function_id,
					      unsigned long arg0,
					      unsigned long arg1,
					      unsigned long arg2,
					      unsigned long arg3,
					      struct arm_smccc_res *res)
{
	arm_smccc_smc(function_id, arg0, arg1, arg2, arg3, 0, 0, 0, res);
}

static inline unsigned long rsi_get_realm_config(struct realm_config *cfg)
{
	struct arm_smccc_res res;

	invoke_rsi_fn_smc_with_res(SMC_RSI_REALM_CONFIG, virt_to_phys(cfg), 0, 0, 0, &res);
	return res.a0;
}

static inline unsigned long rsi_set_addr_range_state(phys_addr_t start,
						     phys_addr_t end,
						     enum ripas state,
						     phys_addr_t *top)
{
	struct arm_smccc_res res;

	invoke_rsi_fn_smc_with_res(SMC_RSI_IPA_STATE_SET,
				   start, (end - start), state, 0, &res);

	*top = res.a1;
	return res.a0;
}

#endif
