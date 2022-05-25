/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#ifndef __ASM_KVM_RME_H
#define __ASM_KVM_RME_H

#include <asm/rmi_smc.h>
#include <uapi/linux/kvm.h>

enum realm_state {
	REALM_STATE_NONE,
	REALM_STATE_NEW,
	REALM_STATE_ACTIVE,
	REALM_STATE_DYING
};

struct realm {
	enum realm_state state;

	void *rd;
	struct realm_params *params;

	/* A spare already delegated page */
	phys_addr_t spare_page;

	unsigned long num_aux;
	unsigned int vmid;
	unsigned int ia_bits;
};

struct rec {
	unsigned long mpidr;
	void *rec_page;
	struct page *aux_pages[REC_PARAMS_AUX_GRANULES];
	struct rec_run *run;
};

int kvm_init_rme(void);
u32 kvm_realm_ipa_limit(void);

int kvm_realm_enable_cap(struct kvm *kvm, struct kvm_enable_cap *cap);
int kvm_init_realm_vm(struct kvm *kvm);
void kvm_destroy_realm(struct kvm *kvm);
void kvm_realm_destroy_rtts(struct realm *realm, u32 ia_bits, u32 start_level);
int kvm_create_rec(struct kvm_vcpu *vcpu);
void kvm_destroy_rec(struct kvm_vcpu *vcpu);

#define RME_RTT_BLOCK_LEVEL	2
#define RME_RTT_MAX_LEVEL	3

#define RME_PAGE_SHIFT		12
#define RME_PAGE_SIZE		BIT(RME_PAGE_SHIFT)
/* See ARM64_HW_PGTABLE_LEVEL_SHIFT() */
#define RME_RTT_LEVEL_SHIFT(l)	\
	((RME_PAGE_SHIFT - 3) * (4 - (l)) + 3)
#define RME_L2_BLOCK_SIZE	BIT(RME_RTT_LEVEL_SHIFT(2))

static inline unsigned long rme_rtt_level_mapsize(int level)
{
	if (WARN_ON(level > RME_RTT_MAX_LEVEL))
		return RME_PAGE_SIZE;

	return (1UL << RME_RTT_LEVEL_SHIFT(level));
}

#endif
