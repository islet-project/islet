/* SPDX-License-Identifier: GPL-2.0-only */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#ifndef __ASM_RSI_H_
#define __ASM_RSI_H_

#include <linux/jump_label.h>
#include <asm/rsi_cmds.h>

extern struct static_key_false rsi_present;

void arm64_setup_memory(void);

void __init arm64_rsi_init(void);
static inline bool is_realm_world(void)
{
	return static_branch_unlikely(&rsi_present);
}

static inline void set_memory_range(phys_addr_t start, phys_addr_t end,
				    enum ripas state)
{
	unsigned long ret;
	phys_addr_t top;

	while (start != end) {
		ret = rsi_set_addr_range_state(start, end, state, &top);
		BUG_ON(ret);
		BUG_ON(top < start);
		BUG_ON(top > end);
		start = top;
	}
}

static inline void set_memory_range_protected(phys_addr_t start, phys_addr_t end)
{
	set_memory_range(start, end, RSI_RIPAS_RAM);
}

static inline void set_memory_range_shared(phys_addr_t start, phys_addr_t end)
{
	set_memory_range(start, end, RSI_RIPAS_EMPTY);
}
#endif
