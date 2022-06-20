/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#ifndef __ASM_KVM_RME_H
#define __ASM_KVM_RME_H

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

int kvm_init_rme(void);
u32 kvm_realm_ipa_limit(void);

int kvm_realm_enable_cap(struct kvm *kvm, struct kvm_enable_cap *cap);
int kvm_init_realm_vm(struct kvm *kvm);
void kvm_destroy_realm(struct kvm *kvm);

#endif
