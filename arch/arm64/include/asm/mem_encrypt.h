/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#ifndef __ASM_MEM_ENCRYPT_H
#define __ASM_MEM_ENCRYPT_H

#include <asm/rsi.h>

/* All DMA must be to non-secure memory for now */
static inline bool force_dma_unencrypted(struct device *dev)
{
	return is_realm_world();
}

int set_memory_encrypted(unsigned long addr, int numpages);
int set_memory_decrypted(unsigned long addr, int numpages);
#endif
