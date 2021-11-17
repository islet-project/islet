// SPDX-License-Identifier: GPL-2.0
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#include <linux/kvm_host.h>

#include <asm/kvm_emulate.h>
#include <asm/kvm_mmu.h>
#include <asm/rmi_cmds.h>
#include <asm/virt.h>

/************ FIXME: Copied from kvm/hyp/pgtable.c **********/
#include <asm/kvm_pgtable.h>

struct kvm_pgtable_walk_data {
	struct kvm_pgtable		*pgt;
	struct kvm_pgtable_walker	*walker;

	u64				addr;
	u64				end;
};

static u32 __kvm_pgd_page_idx(struct kvm_pgtable *pgt, u64 addr)
{
	u64 shift = kvm_granule_shift(pgt->start_level - 1); /* May underflow */
	u64 mask = BIT(pgt->ia_bits) - 1;

	return (addr & mask) >> shift;
}

static u32 kvm_pgd_pages(u32 ia_bits, u32 start_level)
{
	struct kvm_pgtable pgt = {
		.ia_bits	= ia_bits,
		.start_level	= start_level,
	};

	return __kvm_pgd_page_idx(&pgt, -1ULL) + 1;
}

/******************/

static unsigned long rmm_feat_reg0;

static bool rme_supports(unsigned long feature)
{
	return !!u64_get_bits(rmm_feat_reg0, feature);
}

static int rmi_check_version(void)
{
	struct arm_smccc_res res;
	int version_major, version_minor;

	arm_smccc_1_1_invoke(SMC_RMI_VERSION, &res);

	if (res.a0 == SMCCC_RET_NOT_SUPPORTED)
		return -ENXIO;

	version_major = RMI_ABI_VERSION_GET_MAJOR(res.a0);
	version_minor = RMI_ABI_VERSION_GET_MINOR(res.a0);

	if (version_major != RMI_ABI_MAJOR_VERSION) {
		kvm_err("Unsupported RMI ABI (version %d.%d) we support %d\n",
			version_major, version_minor,
			RMI_ABI_MAJOR_VERSION);
		return -ENXIO;
	}

	kvm_info("RMI ABI version %d.%d\n", version_major, version_minor);

	return 0;
}

static unsigned long create_realm_feat_reg0(struct kvm *kvm)
{
	unsigned long ia_bits = VTCR_EL2_IPA(kvm->arch.vtcr);
	u64 feat_reg0 = 0;

	int num_bps = u64_get_bits(rmm_feat_reg0,
				   RMI_FEATURE_REGISTER_0_NUM_BPS);
	int num_wps = u64_get_bits(rmm_feat_reg0,
				   RMI_FEATURE_REGISTER_0_NUM_WPS);

	feat_reg0 |= u64_encode_bits(ia_bits, RMI_FEATURE_REGISTER_0_S2SZ);
	feat_reg0 |= u64_encode_bits(num_bps, RMI_FEATURE_REGISTER_0_NUM_BPS);
	feat_reg0 |= u64_encode_bits(num_wps, RMI_FEATURE_REGISTER_0_NUM_WPS);

	return feat_reg0;
}

u32 kvm_realm_ipa_limit(void)
{
	return u64_get_bits(rmm_feat_reg0, RMI_FEATURE_REGISTER_0_S2SZ);
}

static u32 get_start_level(struct kvm *kvm)
{
	long sl0 = FIELD_GET(VTCR_EL2_SL0_MASK, kvm->arch.vtcr);

	return VTCR_EL2_TGRAN_SL0_BASE - sl0;
}

static int realm_create_rd(struct kvm *kvm)
{
	struct realm *realm = &kvm->arch.realm;
	struct realm_params *params = realm->params;
	void *rd = NULL;
	phys_addr_t rd_phys, params_phys;
	struct kvm_pgtable *pgt = kvm->arch.mmu.pgt;
	unsigned int pgd_sz;
	int i, r;

	if (WARN_ON(realm->rd) || WARN_ON(!realm->params))
		return -EEXIST;

	rd = (void *)__get_free_page(GFP_KERNEL);
	if (!rd)
		return -ENOMEM;

	rd_phys = virt_to_phys(rd);
	if (rmi_granule_delegate(rd_phys)) {
		r = -ENXIO;
		goto out;
	}

	pgd_sz = kvm_pgd_pages(pgt->ia_bits, pgt->start_level);
	for (i = 0; i < pgd_sz; i++) {
		phys_addr_t pgd_phys = kvm->arch.mmu.pgd_phys + i * PAGE_SIZE;

		if (rmi_granule_delegate(pgd_phys)) {
			r = -ENXIO;
			goto out_undelegate_tables;
		}
	}

	params->rtt_level_start = get_start_level(kvm);
	params->rtt_num_start = pgd_sz;
	params->rtt_base = kvm->arch.mmu.pgd_phys;
	params->vmid = realm->vmid;

	params_phys = virt_to_phys(params);

	if (rmi_realm_create(rd_phys, params_phys)) {
		r = -ENXIO;
		goto out_undelegate_tables;
	}

	realm->rd = rd;
	realm->ia_bits = VTCR_EL2_IPA(kvm->arch.vtcr);

	if (WARN_ON(rmi_rec_aux_count(rd_phys, &realm->num_aux))) {
		WARN_ON(rmi_realm_destroy(rd_phys));
		goto out_undelegate_tables;
	}

	return 0;

out_undelegate_tables:
	while (--i >= 0) {
		phys_addr_t pgd_phys = kvm->arch.mmu.pgd_phys + i * PAGE_SIZE;

		WARN_ON(rmi_granule_undelegate(pgd_phys));
	}
	WARN_ON(rmi_granule_undelegate(rd_phys));
out:
	free_page((unsigned long)rd);
	return r;
}

/* Protects access to rme_vmid_bitmap */
static DEFINE_SPINLOCK(rme_vmid_lock);
static unsigned long *rme_vmid_bitmap;

static int rme_vmid_init(void)
{
	unsigned int vmid_count = 1 << kvm_get_vmid_bits();

	rme_vmid_bitmap = bitmap_zalloc(vmid_count, GFP_KERNEL);
	if (!rme_vmid_bitmap) {
		kvm_err("%s: Couldn't allocate rme vmid bitmap\n", __func__);
		return -ENOMEM;
	}

	return 0;
}

static int rme_vmid_reserve(void)
{
	int ret;
	unsigned int vmid_count = 1 << kvm_get_vmid_bits();

	spin_lock(&rme_vmid_lock);
	ret = bitmap_find_free_region(rme_vmid_bitmap, vmid_count, 0);
	spin_unlock(&rme_vmid_lock);

	return ret;
}

static void rme_vmid_release(unsigned int vmid)
{
	spin_lock(&rme_vmid_lock);
	bitmap_release_region(rme_vmid_bitmap, vmid, 0);
	spin_unlock(&rme_vmid_lock);
}

static int kvm_create_realm(struct kvm *kvm)
{
	struct realm *realm = &kvm->arch.realm;
	int ret;

	if (!kvm_is_realm(kvm) || kvm_realm_state(kvm) != REALM_STATE_NONE)
		return -EEXIST;

	ret = rme_vmid_reserve();
	if (ret < 0)
		return ret;
	realm->vmid = ret;

	ret = realm_create_rd(kvm);
	if (ret) {
		rme_vmid_release(realm->vmid);
		return ret;
	}

	WRITE_ONCE(realm->state, REALM_STATE_NEW);

	/* The realm is up, free the parameters.  */
	free_page((unsigned long)realm->params);
	realm->params = NULL;

	return 0;
}

static int config_realm_hash_algo(struct realm *realm,
				  struct kvm_cap_arm_rme_config_item *cfg)
{
	switch (cfg->hash_algo) {
	case KVM_CAP_ARM_RME_MEASUREMENT_ALGO_SHA256:
		if (!rme_supports(RMI_FEATURE_REGISTER_0_HASH_SHA_256))
			return -EINVAL;
		break;
	case KVM_CAP_ARM_RME_MEASUREMENT_ALGO_SHA512:
		if (!rme_supports(RMI_FEATURE_REGISTER_0_HASH_SHA_512))
			return -EINVAL;
		break;
	default:
		return -EINVAL;
	}
	realm->params->measurement_algo = cfg->hash_algo;
	return 0;
}

static int config_realm_sve(struct realm *realm,
			    struct kvm_cap_arm_rme_config_item *cfg)
{
	u64 features_0 = realm->params->features_0;
	int max_sve_vq = u64_get_bits(rmm_feat_reg0,
				      RMI_FEATURE_REGISTER_0_SVE_VL);

	if (!rme_supports(RMI_FEATURE_REGISTER_0_SVE_EN))
		return -EINVAL;

	if (cfg->sve_vq > max_sve_vq)
		return -EINVAL;

	features_0 &= ~(RMI_FEATURE_REGISTER_0_SVE_EN |
			RMI_FEATURE_REGISTER_0_SVE_VL);
	features_0 |= u64_encode_bits(1, RMI_FEATURE_REGISTER_0_SVE_EN);
	features_0 |= u64_encode_bits(cfg->sve_vq,
				      RMI_FEATURE_REGISTER_0_SVE_VL);

	realm->params->features_0 = features_0;
	return 0;
}

static int kvm_rme_config_realm(struct kvm *kvm, struct kvm_enable_cap *cap)
{
	struct kvm_cap_arm_rme_config_item cfg;
	struct realm *realm = &kvm->arch.realm;
	int r = 0;

	if (kvm_realm_state(kvm) != REALM_STATE_NONE)
		return -EBUSY;

	if (copy_from_user(&cfg, (void __user *)cap->args[1], sizeof(cfg)))
		return -EFAULT;

	switch (cfg.cfg) {
	case KVM_CAP_ARM_RME_CFG_RPV:
		memcpy(&realm->params->rpv, &cfg.rpv, sizeof(cfg.rpv));
		break;
	case KVM_CAP_ARM_RME_CFG_HASH_ALGO:
		r = config_realm_hash_algo(realm, &cfg);
		break;
	case KVM_CAP_ARM_RME_CFG_SVE:
		r = config_realm_sve(realm, &cfg);
		break;
	default:
		r = -EINVAL;
	}

	return r;
}

int kvm_realm_enable_cap(struct kvm *kvm, struct kvm_enable_cap *cap)
{
	int r = 0;

	switch (cap->args[0]) {
	case KVM_CAP_ARM_RME_CONFIG_REALM:
		r = kvm_rme_config_realm(kvm, cap);
		break;
	case KVM_CAP_ARM_RME_CREATE_RD:
		if (kvm->created_vcpus) {
			r = -EBUSY;
			break;
		}

		r = kvm_create_realm(kvm);
		break;
	default:
		r = -EINVAL;
		break;
	}

	return r;
}

void kvm_destroy_realm(struct kvm *kvm)
{
	struct realm *realm = &kvm->arch.realm;
	struct kvm_pgtable *pgt = kvm->arch.mmu.pgt;
	unsigned int pgd_sz;
	int i;

	if (realm->params) {
		free_page((unsigned long)realm->params);
		realm->params = NULL;
	}

	if (kvm_realm_state(kvm) == REALM_STATE_NONE)
		return;

	WRITE_ONCE(realm->state, REALM_STATE_DYING);

	rme_vmid_release(realm->vmid);

	if (realm->rd) {
		phys_addr_t rd_phys = virt_to_phys(realm->rd);

		if (WARN_ON(rmi_realm_destroy(rd_phys)))
			return;
		if (WARN_ON(rmi_granule_undelegate(rd_phys)))
			return;
		free_page((unsigned long)realm->rd);
		realm->rd = NULL;
	}

	pgd_sz = kvm_pgd_pages(pgt->ia_bits, pgt->start_level);
	for (i = 0; i < pgd_sz; i++) {
		phys_addr_t pgd_phys = kvm->arch.mmu.pgd_phys + i * PAGE_SIZE;

		if (WARN_ON(rmi_granule_undelegate(pgd_phys)))
			return;
	}

	kvm_free_stage2_pgd(&kvm->arch.mmu);
}

int kvm_init_realm_vm(struct kvm *kvm)
{
	struct realm_params *params;

	params = (struct realm_params *)get_zeroed_page(GFP_KERNEL);
	if (!params)
		return -ENOMEM;

	params->features_0 = create_realm_feat_reg0(kvm);
	kvm->arch.realm.params = params;
	return 0;
}

int kvm_init_rme(void)
{
	int ret;

	if (PAGE_SIZE != SZ_4K)
		/* Only 4k page size on the host is supported */
		return 0;

	if (rmi_check_version())
		/* Continue without realm support */
		return 0;

	ret = rme_vmid_init();
	if (ret)
		return ret;

	WARN_ON(rmi_features(0, &rmm_feat_reg0));

	/* Future patch will enable static branch kvm_rme_is_available */

	return 0;
}
