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

static void realm_destroy_undelegate_range(struct realm *realm,
					   unsigned long ipa,
					   unsigned long addr,
					   ssize_t size)
{
	unsigned long rd = virt_to_phys(realm->rd);
	int ret;

	while (size > 0) {
		ret = rmi_data_destroy(rd, ipa);
		WARN_ON(ret);
		ret = rmi_granule_undelegate(addr);

		if (ret)
			get_page(phys_to_page(addr));

		addr += PAGE_SIZE;
		ipa += PAGE_SIZE;
		size -= PAGE_SIZE;
	}
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
	realm->spare_page = PHYS_ADDR_MAX;
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

static int realm_rtt_destroy(struct realm *realm, unsigned long addr,
			     int level, phys_addr_t rtt_granule)
{
	addr = ALIGN_DOWN(addr, rme_rtt_level_mapsize(level - 1));
	return rmi_rtt_destroy(rtt_granule, virt_to_phys(realm->rd), addr,
			level);
}

static int realm_destroy_free_rtt(struct realm *realm, unsigned long addr,
				  int level, phys_addr_t rtt_granule)
{
	if (realm_rtt_destroy(realm, addr, level, rtt_granule))
		return -ENXIO;
	if (!WARN_ON(rmi_granule_undelegate(rtt_granule)))
		put_page(phys_to_page(rtt_granule));

	return 0;
}

static int realm_rtt_create(struct realm *realm,
			    unsigned long addr,
			    int level,
			    phys_addr_t phys)
{
	addr = ALIGN_DOWN(addr, rme_rtt_level_mapsize(level - 1));
	return rmi_rtt_create(phys, virt_to_phys(realm->rd), addr, level);
}

static int realm_tear_down_rtt_range(struct realm *realm, int level,
				     unsigned long start, unsigned long end)
{
	phys_addr_t rd = virt_to_phys(realm->rd);
	ssize_t map_size = rme_rtt_level_mapsize(level);
	unsigned long addr, next_addr;
	bool failed = false;

	for (addr = start; addr < end; addr = next_addr) {
		phys_addr_t rtt_addr, tmp_rtt;
		struct rtt_entry rtt;
		unsigned long end_addr;

		next_addr = ALIGN(addr + 1, map_size);

		end_addr = min(next_addr, end);

		if (rmi_rtt_read_entry(rd, ALIGN_DOWN(addr, map_size),
				       level, &rtt)) {
			failed = true;
			continue;
		}

		rtt_addr = rmi_rtt_get_phys(&rtt);
		WARN_ON(level != rtt.walk_level);

		switch (rtt.state) {
		case RMI_UNASSIGNED:
		case RMI_DESTROYED:
			break;
		case RMI_TABLE:
			if (realm_tear_down_rtt_range(realm, level + 1,
						      addr, end_addr)) {
				failed = true;
				break;
			}
			if (IS_ALIGNED(addr, map_size) &&
			    next_addr <= end &&
			    realm_destroy_free_rtt(realm, addr, level + 1,
						   rtt_addr))
				failed = true;
			break;
		case RMI_ASSIGNED:
			WARN_ON(!rtt_addr);
			/*
			 * If there is a block mapping, break it now, using the
			 * spare_page. We are sure to have a valid delegated
			 * page at spare_page before we enter here, otherwise
			 * WARN once, which will be followed by further
			 * warnings.
			 */
			tmp_rtt = realm->spare_page;
			if (level == 2 &&
			    !WARN_ON_ONCE(tmp_rtt == PHYS_ADDR_MAX) &&
			    realm_rtt_create(realm, addr,
					     RME_RTT_MAX_LEVEL, tmp_rtt)) {
				WARN_ON(1);
				failed = true;
				break;
			}
			realm_destroy_undelegate_range(realm, addr,
						       rtt_addr, map_size);
			/*
			 * Collapse the last level table and make the spare page
			 * reusable again.
			 */
			if (level == 2 &&
			    realm_rtt_destroy(realm, addr, RME_RTT_MAX_LEVEL,
					      tmp_rtt))
				failed = true;
			break;
		case RMI_VALID_NS:
			WARN_ON(rmi_rtt_unmap_unprotected(rd, addr, level));
			break;
		default:
			WARN_ON(1);
			failed = true;
			break;
		}
	}

	return failed ? -EINVAL : 0;
}

void kvm_realm_destroy_rtts(struct realm *realm, u32 ia_bits, u32 start_level)
{
	realm_tear_down_rtt_range(realm, start_level, 0, (1UL << ia_bits));
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
	if (realm->spare_page != PHYS_ADDR_MAX) {
		if (!WARN_ON(rmi_granule_undelegate(realm->spare_page)))
			free_page((unsigned long)phys_to_virt(realm->spare_page));
		realm->spare_page = PHYS_ADDR_MAX;
	}

	pgd_sz = kvm_pgd_pages(pgt->ia_bits, pgt->start_level);
	for (i = 0; i < pgd_sz; i++) {
		phys_addr_t pgd_phys = kvm->arch.mmu.pgd_phys + i * PAGE_SIZE;

		if (WARN_ON(rmi_granule_undelegate(pgd_phys)))
			return;
	}

	kvm_free_stage2_pgd(&kvm->arch.mmu);
}

static void free_rec_aux(struct page **aux_pages,
			 unsigned int num_aux)
{
	unsigned int i;

	for (i = 0; i < num_aux; i++) {
		phys_addr_t aux_page_phys = page_to_phys(aux_pages[i]);

		if (WARN_ON(rmi_granule_undelegate(aux_page_phys)))
			continue;

		__free_page(aux_pages[i]);
	}
}

static int alloc_rec_aux(struct page **aux_pages,
			 u64 *aux_phys_pages,
			 unsigned int num_aux)
{
	int ret;
	unsigned int i;

	for (i = 0; i < num_aux; i++) {
		struct page *aux_page;
		phys_addr_t aux_page_phys;

		aux_page = alloc_page(GFP_KERNEL);
		if (!aux_page) {
			ret = -ENOMEM;
			goto out_err;
		}
		aux_page_phys = page_to_phys(aux_page);
		if (rmi_granule_delegate(aux_page_phys)) {
			__free_page(aux_page);
			ret = -ENXIO;
			goto out_err;
		}
		aux_pages[i] = aux_page;
		aux_phys_pages[i] = aux_page_phys;
	}

	return 0;
out_err:
	free_rec_aux(aux_pages, i);
	return ret;
}

int kvm_create_rec(struct kvm_vcpu *vcpu)
{
	struct user_pt_regs *vcpu_regs = vcpu_gp_regs(vcpu);
	unsigned long mpidr = kvm_vcpu_get_mpidr_aff(vcpu);
	struct realm *realm = &vcpu->kvm->arch.realm;
	struct rec *rec = &vcpu->arch.rec;
	unsigned long rec_page_phys;
	struct rec_params *params;
	int r, i;

	if (kvm_realm_state(vcpu->kvm) != REALM_STATE_NEW)
		return -ENOENT;

	/*
	 * The RMM will report PSCI v1.0 to Realms and the KVM_ARM_VCPU_PSCI_0_2
	 * flag covers v0.2 and onwards.
	 */
	if (!test_bit(KVM_ARM_VCPU_PSCI_0_2, vcpu->arch.features))
		return -EINVAL;

	BUILD_BUG_ON(sizeof(*params) > PAGE_SIZE);
	BUILD_BUG_ON(sizeof(*rec->run) > PAGE_SIZE);

	params = (struct rec_params *)get_zeroed_page(GFP_KERNEL);
	rec->rec_page = (void *)__get_free_page(GFP_KERNEL);
	rec->run = (void *)get_zeroed_page(GFP_KERNEL);
	if (!params || !rec->rec_page || !rec->run) {
		r = -ENOMEM;
		goto out_free_pages;
	}

	for (i = 0; i < ARRAY_SIZE(params->gprs); i++)
		params->gprs[i] = vcpu_regs->regs[i];

	params->pc = vcpu_regs->pc;

	if (vcpu->vcpu_id == 0)
		params->flags |= REC_PARAMS_FLAG_RUNNABLE;

	rec_page_phys = virt_to_phys(rec->rec_page);

	if (rmi_granule_delegate(rec_page_phys)) {
		r = -ENXIO;
		goto out_free_pages;
	}

	r = alloc_rec_aux(rec->aux_pages, params->aux, realm->num_aux);
	if (r)
		goto out_undelegate_rmm_rec;

	params->num_rec_aux = realm->num_aux;
	params->mpidr = mpidr;

	if (rmi_rec_create(rec_page_phys,
			   virt_to_phys(realm->rd),
			   virt_to_phys(params))) {
		r = -ENXIO;
		goto out_free_rec_aux;
	}

	rec->mpidr = mpidr;

	free_page((unsigned long)params);
	return 0;

out_free_rec_aux:
	free_rec_aux(rec->aux_pages, realm->num_aux);
out_undelegate_rmm_rec:
	if (WARN_ON(rmi_granule_undelegate(rec_page_phys)))
		rec->rec_page = NULL;
out_free_pages:
	free_page((unsigned long)rec->run);
	free_page((unsigned long)rec->rec_page);
	free_page((unsigned long)params);
	return r;
}

void kvm_destroy_rec(struct kvm_vcpu *vcpu)
{
	struct realm *realm = &vcpu->kvm->arch.realm;
	struct rec *rec = &vcpu->arch.rec;
	unsigned long rec_page_phys;

	if (!vcpu_is_rec(vcpu))
		return;

	rec_page_phys = virt_to_phys(rec->rec_page);

	if (WARN_ON(rmi_rec_destroy(rec_page_phys)))
		return;
	if (WARN_ON(rmi_granule_undelegate(rec_page_phys)))
		return;

	free_rec_aux(rec->aux_pages, realm->num_aux);
	free_page((unsigned long)rec->rec_page);
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
