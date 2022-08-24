// SPDX-License-Identifier: GPL-2.0-only
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#include <linux/kvm_host.h>
#include <kvm/arm_hypercalls.h>
#include <kvm/arm_psci.h>

#include <asm/rmi_smc.h>
#include <asm/kvm_emulate.h>
#include <asm/kvm_rme.h>
#include <asm/kvm_mmu.h>

typedef int (*exit_handler_fn)(struct kvm_vcpu *vcpu);

static int rec_exit_reason_notimpl(struct kvm_vcpu *vcpu)
{
	struct rec *rec = &vcpu->arch.rec;

	pr_err("[vcpu %d] Unhandled exit reason from realm (ESR: %#llx)\n",
	       vcpu->vcpu_id, rec->run->exit.esr);
	return -ENXIO;
}

static int rec_exit_sync_dabt(struct kvm_vcpu *vcpu)
{
	struct rec *rec = &vcpu->arch.rec;

	if (kvm_vcpu_dabt_iswrite(vcpu) && kvm_vcpu_dabt_isvalid(vcpu))
		vcpu_set_reg(vcpu, kvm_vcpu_dabt_get_rd(vcpu),
			     rec->run->exit.gprs[0]);

	return kvm_handle_guest_abort(vcpu);
}

static int rec_exit_sync_iabt(struct kvm_vcpu *vcpu)
{
	struct rec *rec = &vcpu->arch.rec;

	pr_err("[vcpu %d] Unhandled instruction abort (ESR: %#llx).\n",
	       vcpu->vcpu_id, rec->run->exit.esr);
	return -ENXIO;
}

static int rec_exit_sys_reg(struct kvm_vcpu *vcpu)
{
	struct rec *rec = &vcpu->arch.rec;
	unsigned long esr = kvm_vcpu_get_esr(vcpu);
	int rt = kvm_vcpu_sys_get_rt(vcpu);
	bool is_write = !(esr & 1);
	int ret;

	if (is_write)
		vcpu_set_reg(vcpu, rt, rec->run->exit.gprs[0]);

	ret = kvm_handle_sys_reg(vcpu);

	if (ret >= 0 && !is_write)
		rec->run->entry.gprs[0] = vcpu_get_reg(vcpu, rt);

	return ret;
}

static exit_handler_fn rec_exit_handlers[] = {
	[0 ... ESR_ELx_EC_MAX]	= rec_exit_reason_notimpl,
	[ESR_ELx_EC_SYS64]	= rec_exit_sys_reg,
	[ESR_ELx_EC_DABT_LOW]	= rec_exit_sync_dabt,
	[ESR_ELx_EC_IABT_LOW]	= rec_exit_sync_iabt
};

static int rec_exit_psci(struct kvm_vcpu *vcpu)
{
	struct rec *rec = &vcpu->arch.rec;
	int i;

	for (i = 0; i < REC_RUN_GPRS; i++)
		vcpu_set_reg(vcpu, i, rec->run->exit.gprs[i]);

	return kvm_psci_call(vcpu);
}

static int rec_exit_ripas_change(struct kvm_vcpu *vcpu)
{
	struct realm *realm = &vcpu->kvm->arch.realm;
	struct rec *rec = &vcpu->arch.rec;
	unsigned long base = rec->run->exit.ripas_base;
	unsigned long size = rec->run->exit.ripas_size;
	unsigned long ripas = rec->run->exit.ripas_value & 1;
	int ret = -EINVAL;

	if (realm_is_addr_protected(realm, base) &&
	    realm_is_addr_protected(realm, base + size))
		ret = realm_set_ipa_state(vcpu, base, base + size, ripas);

	WARN(ret, "Unable to satisfy SET_IPAS for %#lx - %#lx, ripas: %#lx\n",
	     base, base + size, ripas);

	return 1;
}

static int rec_exit_host_call(struct kvm_vcpu *vcpu)
{
	int ret, i;
	struct rec *rec = &vcpu->arch.rec;

	vcpu->stat.hvc_exit_stat++;

	for (i = 0; i < REC_RUN_GPRS; i++)
		vcpu_set_reg(vcpu, i, rec->run->exit.gprs[i]);

	ret = kvm_hvc_call_handler(vcpu);

	if (ret < 0) {
		vcpu_set_reg(vcpu, 0, ~0UL);
		ret = 1;
	}

	for (i = 0; i < REC_RUN_GPRS; i++)
		rec->run->entry.gprs[i] = vcpu_get_reg(vcpu, i);

	return ret;
}

static void update_arch_timer_irq_lines(struct kvm_vcpu *vcpu)
{
	struct rec *rec = &vcpu->arch.rec;

	__vcpu_sys_reg(vcpu, CNTV_CTL_EL0) = rec->run->exit.cntv_ctl;
	__vcpu_sys_reg(vcpu, CNTV_CVAL_EL0) = rec->run->exit.cntv_cval;
	__vcpu_sys_reg(vcpu, CNTP_CTL_EL0) = rec->run->exit.cntp_ctl;
	__vcpu_sys_reg(vcpu, CNTP_CVAL_EL0) = rec->run->exit.cntp_cval;

	kvm_realm_timers_update(vcpu);
}

/*
 * Return > 0 to return to guest, < 0 on error, 0 (and set exit_reason) on
 * proper exit to userspace.
 */
int handle_rme_exit(struct kvm_vcpu *vcpu, int rec_run_ret)
{
	struct rec *rec = &vcpu->arch.rec;
	u8 esr_ec = ESR_ELx_EC(rec->run->exit.esr);
	unsigned long status, index;

	status = RMI_RETURN_STATUS(rec_run_ret);
	index = RMI_RETURN_INDEX(rec_run_ret);

	/*
	 * If a PSCI_SYSTEM_OFF request raced with a vcpu executing, we might
	 * see the following status code and index indicating an attempt to run
	 * a REC when the RD state is SYSTEM_OFF.  In this case, we just need to
	 * return to user space which can deal with the system event or will try
	 * to run the KVM VCPU again, at which point we will no longer attempt
	 * to enter the Realm because we will have a sleep request pending on
	 * the VCPU as a result of KVM's PSCI handling.
	 */
	if (status == RMI_ERROR_REALM && index == 1) {
		vcpu->run->exit_reason = KVM_EXIT_UNKNOWN;
		return 0;
	}

	if (rec_run_ret)
		return -ENXIO;

	vcpu->arch.fault.esr_el2 = rec->run->exit.esr;
	vcpu->arch.fault.far_el2 = rec->run->exit.far;
	vcpu->arch.fault.hpfar_el2 = rec->run->exit.hpfar;

	update_arch_timer_irq_lines(vcpu);

	/* Reset the emulation flags for the next run of the REC */
	rec->run->entry.flags = 0;

	switch (rec->run->exit.exit_reason) {
	case RMI_EXIT_SYNC:
		return rec_exit_handlers[esr_ec](vcpu);
	case RMI_EXIT_IRQ:
	case RMI_EXIT_FIQ:
		return 1;
	case RMI_EXIT_PSCI:
		return rec_exit_psci(vcpu);
	case RMI_EXIT_RIPAS_CHANGE:
		return rec_exit_ripas_change(vcpu);
	case RMI_EXIT_HOST_CALL:
		return rec_exit_host_call(vcpu);
	}

	kvm_pr_unimpl("Unsupported exit reason: %u\n",
		      rec->run->exit.exit_reason);
	vcpu->run->exit_reason = KVM_EXIT_INTERNAL_ERROR;
	return 0;
}
