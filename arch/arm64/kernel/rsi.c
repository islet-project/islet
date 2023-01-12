// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#include <linux/jump_label.h>
#include <linux/memblock.h>
#include <asm/rsi.h>

struct realm_config __attribute((aligned(PAGE_SIZE))) config;

unsigned long prot_ns_shared;
EXPORT_SYMBOL(prot_ns_shared);

unsigned int phys_mask_shift = CONFIG_ARM64_PA_BITS;

DEFINE_STATIC_KEY_FALSE_RO(rsi_present);

static bool rsi_version_matches(void)
{
	unsigned long ver = rsi_get_version();

	if (ver == SMCCC_RET_NOT_SUPPORTED)
		return false;

	pr_info("RME: RSI version %lu.%lu advertised\n",
		RSI_ABI_VERSION_GET_MAJOR(ver),
		RSI_ABI_VERSION_GET_MINOR(ver));

	return (ver >= RSI_ABI_VERSION &&
		RSI_ABI_VERSION_GET_MAJOR(ver) == RSI_ABI_VERSION_MAJOR);
}

void arm64_setup_memory(void)
{
	u64 i;
	phys_addr_t start, end;

	if (!static_branch_unlikely(&rsi_present))
		return;

	/*
	 * Iterate over the available memory ranges
	 * and convert the state to protected memory.
	 */
	for_each_mem_range(i, &start, &end) {
		set_memory_range_protected(start, end);
	}
}

void __init arm64_rsi_init(void)
{
	if (!rsi_version_matches())
		return;
	if (rsi_get_realm_config(&config))
		return;
	prot_ns_shared = BIT(config.ipa_bits - 1);

	if (config.ipa_bits - 1 < phys_mask_shift)
		phys_mask_shift = config.ipa_bits - 1;

	static_branch_enable(&rsi_present);
}
