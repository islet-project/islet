/* SPDX-License-Identifier: GPL-2.0 */
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#ifndef __ASM_KVM_RME_H
#define __ASM_KVM_RME_H

enum realm_state {
	REALM_STATE_NONE,
	REALM_STATE_NEW,
	REALM_STATE_ACTIVE,
	REALM_STATE_DYING
};

struct realm {
	enum realm_state state;
};

int kvm_init_rme(void);

#endif
