// SPDX-License-Identifier: GPL-2.0
/*
 * Copyright (C) 2023 ARM Ltd.
 */

#include <linux/kvm_host.h>
#include <linux/hugetlb.h>

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

bool kvm_rme_supports_sve(void)
{
	return rme_supports(RMI_FEATURE_REGISTER_0_SVE_EN);
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

#ifdef PROTOTYPE_RMI_ABI_MAJOR_VERSION
	// Support the prototype
	if (version_major == PROTOTYPE_RMI_ABI_MAJOR_VERSION) {
		kvm_err("Using prototype RMM support (version %d.%d)\n",
			version_major, version_minor);
		return 0;
	}
#endif
	if (version_major != RMI_ABI_MAJOR_VERSION) {
		kvm_err("Unsupported RMI ABI (version %d.%d) we support %d\n",
			version_major, version_minor,
			RMI_ABI_MAJOR_VERSION);
		return -ENXIO;
	}

	kvm_info("RMI ABI version %d.%d\n", version_major, version_minor);

	return 0;
}

static phys_addr_t __alloc_delegated_page(struct realm *realm,
					  struct kvm_mmu_memory_cache *mc, gfp_t flags)
{
	phys_addr_t phys = PHYS_ADDR_MAX;
	void *virt;

	if (realm->spare_page != PHYS_ADDR_MAX) {
		swap(realm->spare_page, phys);
		goto out;
	}

	if (mc)
		virt = kvm_mmu_memory_cache_alloc(mc);
	else
		virt = (void *)__get_free_page(flags);

	if (!virt)
		goto out;

	phys = virt_to_phys(virt);

	if (rmi_granule_delegate(phys)) {
		free_page((unsigned long)virt);

		phys = PHYS_ADDR_MAX;
	}

out:
	return phys;
}

static phys_addr_t alloc_delegated_page(struct realm *realm,
					struct kvm_mmu_memory_cache *mc)
{
	return __alloc_delegated_page(realm, mc, GFP_KERNEL);
}

static void free_delegated_page(struct realm *realm, phys_addr_t phys)
{
	if (realm->spare_page == PHYS_ADDR_MAX) {
		realm->spare_page = phys;
		return;
	}

	if (WARN_ON(rmi_granule_undelegate(phys))) {
		/* Undelegate failed: leak the page */
		return;
	}

	free_page((unsigned long)phys_to_virt(phys));
}

int realm_psci_complete(struct kvm_vcpu *calling, struct kvm_vcpu *target)
{
	int ret;

	ret = rmi_psci_complete(virt_to_phys(calling->arch.rec.rec_page),
				virt_to_phys(target->arch.rec.rec_page));

	if (ret)
		return -EINVAL;

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

static int realm_create_rtt_levels(struct realm *realm,
				   unsigned long ipa,
				   int level,
				   int max_level,
				   struct kvm_mmu_memory_cache *mc)
{
	if (WARN_ON(level == max_level))
		return 0;

	while (level++ < max_level) {
		phys_addr_t rtt = alloc_delegated_page(realm, mc);

		if (rtt == PHYS_ADDR_MAX)
			return -ENOMEM;

		if (realm_rtt_create(realm, ipa, level, rtt)) {
			free_delegated_page(realm, rtt);
			return -ENXIO;
		}
	}

	return 0;
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

void kvm_realm_unmap_range(struct kvm *kvm, unsigned long ipa, u64 size)
{
	u32 ia_bits = kvm->arch.mmu.pgt->ia_bits;
	u32 start_level = kvm->arch.mmu.pgt->start_level;
	unsigned long end = ipa + size;
	struct realm *realm = &kvm->arch.realm;
	phys_addr_t tmp_rtt = PHYS_ADDR_MAX;

	if (end > (1UL << ia_bits))
		end = 1UL << ia_bits;
	/*
	 * Make sure we have a spare delegated page for tearing down the
	 * block mappings. We must use Atomic allocations as we are called
	 * with kvm->mmu_lock held.
	 */
	if (realm->spare_page == PHYS_ADDR_MAX) {
		tmp_rtt = __alloc_delegated_page(realm, NULL, GFP_ATOMIC);
		/*
		 * We don't have to check the status here, as we may not
		 * have a block level mapping. Delay any error to the point
		 * where we need it.
		 */
		realm->spare_page = tmp_rtt;
	}

	realm_tear_down_rtt_range(&kvm->arch.realm, start_level, ipa, end);

	/* Free up the atomic page, if there were any */
	if (tmp_rtt != PHYS_ADDR_MAX) {
		free_delegated_page(realm, tmp_rtt);
		/*
		 * Update the spare_page after we have freed the
		 * above page to make sure it doesn't get cached
		 * in spare_page.
		 * We should re-write this part and always have
		 * a dedicated page for handling block mappings.
		 */
		realm->spare_page = PHYS_ADDR_MAX;
	}
}

static int realm_create_protected_data_page(struct realm *realm,
					    unsigned long ipa,
					    struct page *dst_page,
					    struct page *tmp_page)
{
	phys_addr_t dst_phys, tmp_phys;
	int ret;

	copy_page(page_address(tmp_page), page_address(dst_page));

	dst_phys = page_to_phys(dst_page);
	tmp_phys = page_to_phys(tmp_page);

	if (rmi_granule_delegate(dst_phys))
		return -ENXIO;

	ret = rmi_data_create(dst_phys, virt_to_phys(realm->rd), ipa, tmp_phys,
			      RMI_MEASURE_CONTENT);

	if (RMI_RETURN_STATUS(ret) == RMI_ERROR_RTT) {
		/* Create missing RTTs and retry */
		int level = RMI_RETURN_INDEX(ret);

		ret = realm_create_rtt_levels(realm, ipa, level,
					      RME_RTT_MAX_LEVEL, NULL);
		if (ret)
			goto err;

		ret = rmi_data_create(dst_phys, virt_to_phys(realm->rd), ipa,
				      tmp_phys, RMI_MEASURE_CONTENT);
	}

	if (ret)
		goto err;

	return 0;

err:
	if (WARN_ON(rmi_granule_undelegate(dst_phys))) {
		/* Page can't be returned to NS world so is lost */
		get_page(dst_page);
	}
	return -ENXIO;
}

static int fold_rtt(phys_addr_t rd, unsigned long addr, int level,
		    struct realm *realm)
{
	struct rtt_entry rtt;
	phys_addr_t rtt_addr;

	if (rmi_rtt_read_entry(rd, addr, level, &rtt))
		return -ENXIO;

	if (rtt.state != RMI_TABLE)
		return -EINVAL;

	rtt_addr = rmi_rtt_get_phys(&rtt);
	if (rmi_rtt_fold(rtt_addr, rd, addr, level + 1))
		return -ENXIO;

	free_delegated_page(realm, rtt_addr);

	return 0;
}

int realm_map_protected(struct realm *realm,
			unsigned long hva,
			unsigned long base_ipa,
			struct page *dst_page,
			unsigned long map_size,
			struct kvm_mmu_memory_cache *memcache)
{
	phys_addr_t dst_phys = page_to_phys(dst_page);
	phys_addr_t rd = virt_to_phys(realm->rd);
	unsigned long phys = dst_phys;
	unsigned long ipa = base_ipa;
	unsigned long size;
	int map_level;
	int ret = 0;

	if (WARN_ON(!IS_ALIGNED(ipa, map_size)))
		return -EINVAL;

	switch (map_size) {
	case PAGE_SIZE:
		map_level = 3;
		break;
	case RME_L2_BLOCK_SIZE:
		map_level = 2;
		break;
	default:
		return -EINVAL;
	}

	if (map_level < RME_RTT_MAX_LEVEL) {
		/*
		 * A temporary RTT is needed during the map, precreate it,
		 * however if there is an error (e.g. missing parent tables)
		 * this will be handled below.
		 */
		realm_create_rtt_levels(realm, ipa, map_level,
					RME_RTT_MAX_LEVEL, memcache);
	}

	for (size = 0; size < map_size; size += PAGE_SIZE) {
		if (rmi_granule_delegate(phys)) {
			struct rtt_entry rtt;

			/*
			 * It's possible we raced with another VCPU on the same
			 * fault. If the entry exists and matches then exit
			 * early and assume the other VCPU will handle the
			 * mapping.
			 */
			if (rmi_rtt_read_entry(rd, ipa, RME_RTT_MAX_LEVEL, &rtt))
				goto err;

			// FIXME: For a block mapping this could race at level
			// 2 or 3...
			if (WARN_ON((rtt.walk_level != RME_RTT_MAX_LEVEL ||
				     rtt.state != RMI_ASSIGNED ||
				     rtt.desc != phys))) {
				goto err;
			}

			return 0;
		}

		ret = rmi_data_create_unknown(phys, rd, ipa);

		if (RMI_RETURN_STATUS(ret) == RMI_ERROR_RTT) {
			/* Create missing RTTs and retry */
			int level = RMI_RETURN_INDEX(ret);

			ret = realm_create_rtt_levels(realm, ipa, level,
						      RME_RTT_MAX_LEVEL,
						      memcache);
			WARN_ON(ret);
			if (ret)
				goto err_undelegate;

			ret = rmi_data_create_unknown(phys, rd, ipa);
		}
		WARN_ON(ret);

		if (ret)
			goto err_undelegate;

		phys += PAGE_SIZE;
		ipa += PAGE_SIZE;
	}

	if (map_size == RME_L2_BLOCK_SIZE)
		ret = fold_rtt(rd, base_ipa, map_level, realm);
	if (WARN_ON(ret))
		goto err;

	return 0;

err_undelegate:
	if (WARN_ON(rmi_granule_undelegate(phys))) {
		/* Page can't be returned to NS world so is lost */
		get_page(phys_to_page(phys));
	}
err:
	while (size > 0) {
		phys -= PAGE_SIZE;
		size -= PAGE_SIZE;
		ipa -= PAGE_SIZE;

		rmi_data_destroy(rd, ipa);

		if (WARN_ON(rmi_granule_undelegate(phys))) {
			/* Page can't be returned to NS world so is lost */
			get_page(phys_to_page(phys));
		}
	}
	return -ENXIO;
}

int realm_map_non_secure(struct realm *realm,
			 unsigned long ipa,
			 struct page *page,
			 unsigned long map_size,
			 struct kvm_mmu_memory_cache *memcache)
{
	phys_addr_t rd = virt_to_phys(realm->rd);
	int map_level;
	int ret = 0;
	unsigned long desc = page_to_phys(page) |
			     PTE_S2_MEMATTR(MT_S2_FWB_NORMAL) |
			     /* FIXME: Read+Write permissions for now */
			     (3 << 6) |
			     PTE_SHARED;

	if (WARN_ON(!IS_ALIGNED(ipa, map_size)))
		return -EINVAL;

	switch (map_size) {
	case PAGE_SIZE:
		map_level = 3;
		break;
	case RME_L2_BLOCK_SIZE:
		map_level = 2;
		break;
	default:
		return -EINVAL;
	}

	ret = rmi_rtt_map_unprotected(rd, ipa, map_level, desc);

	if (RMI_RETURN_STATUS(ret) == RMI_ERROR_RTT) {
		/* Create missing RTTs and retry */
		int level = RMI_RETURN_INDEX(ret);

		ret = realm_create_rtt_levels(realm, ipa, level, map_level,
					      memcache);
		if (WARN_ON(ret))
			return -ENXIO;

		ret = rmi_rtt_map_unprotected(rd, ipa, map_level, desc);
	}
	if (WARN_ON(ret))
		return -ENXIO;

	return 0;
}

static int populate_par_region(struct kvm *kvm,
			       phys_addr_t ipa_base,
			       phys_addr_t ipa_end)
{
	struct realm *realm = &kvm->arch.realm;
	struct kvm_memory_slot *memslot;
	gfn_t base_gfn, end_gfn;
	int idx;
	phys_addr_t ipa;
	int ret = 0;
	struct page *tmp_page;
	phys_addr_t rd = virt_to_phys(realm->rd);

	base_gfn = gpa_to_gfn(ipa_base);
	end_gfn = gpa_to_gfn(ipa_end);

	idx = srcu_read_lock(&kvm->srcu);
	memslot = gfn_to_memslot(kvm, base_gfn);
	if (!memslot) {
		ret = -EFAULT;
		goto out;
	}

	/* We require the region to be contained within a single memslot */
	if (memslot->base_gfn + memslot->npages < end_gfn) {
		ret = -EINVAL;
		goto out;
	}

	tmp_page = alloc_page(GFP_KERNEL);
	if (!tmp_page) {
		ret = -ENOMEM;
		goto out;
	}

	mmap_read_lock(current->mm);

	ipa = ipa_base;

	while (ipa < ipa_end) {
		struct vm_area_struct *vma;
		unsigned long map_size;
		unsigned int vma_shift;
		unsigned long offset;
		unsigned long hva;
		struct page *page;
		kvm_pfn_t pfn;
		int level;

		hva = gfn_to_hva_memslot(memslot, gpa_to_gfn(ipa));
		vma = vma_lookup(current->mm, hva);
		if (!vma) {
			ret = -EFAULT;
			break;
		}

		// FIXME: To avoid the overmapping issue (see below comment)
		// force the use of 4k pages
		if (is_vm_hugetlb_page(vma) && 0)
			vma_shift = huge_page_shift(hstate_vma(vma));
		else
			vma_shift = PAGE_SHIFT;

		map_size = 1 << vma_shift;

		/*
		 * FIXME: This causes over mapping, but there's no good
		 * solution here with the ABI as it stands
		 */
		ipa = ALIGN_DOWN(ipa, map_size);

		switch (map_size) {
		case RME_L2_BLOCK_SIZE:
			level = 2;
			break;
		case PAGE_SIZE:
			level = 3;
			break;
		default:
			WARN_ONCE(1, "Unsupport vma_shift %d", vma_shift);
			ret = -EFAULT;
			break;
		}

		pfn = gfn_to_pfn_memslot(memslot, gpa_to_gfn(ipa));

		if (is_error_pfn(pfn)) {
			ret = -EFAULT;
			break;
		}

		ret = rmi_rtt_init_ripas(rd, ipa, level);
		if (RMI_RETURN_STATUS(ret) == RMI_ERROR_RTT) {
			ret = realm_create_rtt_levels(realm, ipa,
						      RMI_RETURN_INDEX(ret),
						      level, NULL);
			if (ret)
				break;
			ret = rmi_rtt_init_ripas(rd, ipa, level);
			if (ret) {
				ret = -ENXIO;
				break;
			}
		}

		if (level < RME_RTT_MAX_LEVEL) {
			/*
			 * A temporary RTT is needed during the map, precreate
			 * it, however if there is an error (e.g. missing
			 * parent tables) this will be handled in the
			 * realm_create_protected_data_page() call.
			 */
			realm_create_rtt_levels(realm, ipa, level,
						RME_RTT_MAX_LEVEL, NULL);
		}

		page = pfn_to_page(pfn);

		for (offset = 0; offset < map_size && !ret;
		     offset += PAGE_SIZE, page++) {
			phys_addr_t page_ipa = ipa + offset;

			ret = realm_create_protected_data_page(realm, page_ipa,
							       page, tmp_page);
		}
		if (ret)
			goto err_release_pfn;

		if (level == 2) {
			ret = fold_rtt(rd, ipa, level, realm);
			if (ret)
				goto err_release_pfn;
		}

		ipa += map_size;
		kvm_set_pfn_accessed(pfn);
		kvm_set_pfn_dirty(pfn);
		kvm_release_pfn_dirty(pfn);
err_release_pfn:
		if (ret) {
			kvm_release_pfn_clean(pfn);
			break;
		}
	}

	mmap_read_unlock(current->mm);
	__free_page(tmp_page);

out:
	srcu_read_unlock(&kvm->srcu, idx);
	return ret;
}

static int kvm_populate_realm(struct kvm *kvm,
			      struct kvm_cap_arm_rme_populate_realm_args *args)
{
	phys_addr_t ipa_base, ipa_end;

	if (kvm_realm_state(kvm) != REALM_STATE_NEW)
		return -EBUSY;

	if (!IS_ALIGNED(args->populate_ipa_base, PAGE_SIZE) ||
	    !IS_ALIGNED(args->populate_ipa_size, PAGE_SIZE))
		return -EINVAL;

	ipa_base = args->populate_ipa_base;
	ipa_end = ipa_base + args->populate_ipa_size;

	if (ipa_end < ipa_base)
		return -EINVAL;

	return populate_par_region(kvm, ipa_base, ipa_end);
}

static int set_ipa_state(struct kvm_vcpu *vcpu,
			 unsigned long ipa,
			 unsigned long end,
			 int level,
			 unsigned long ripas)
{
	struct kvm *kvm = vcpu->kvm;
	struct realm *realm = &kvm->arch.realm;
	struct rec *rec = &vcpu->arch.rec;
	phys_addr_t rd_phys = virt_to_phys(realm->rd);
	phys_addr_t rec_phys = virt_to_phys(rec->rec_page);
	unsigned long map_size = rme_rtt_level_mapsize(level);
	int ret;

	while (ipa < end) {
		ret = rmi_rtt_set_ripas(rd_phys, rec_phys, ipa, level, ripas);

		if (!ret) {
			if (!ripas)
				kvm_realm_unmap_range(kvm, ipa, map_size);
		} else if (RMI_RETURN_STATUS(ret) == RMI_ERROR_RTT) {
			int walk_level = RMI_RETURN_INDEX(ret);

			if (walk_level < level) {
				ret = realm_create_rtt_levels(realm, ipa,
							      walk_level,
							      level, NULL);
				if (ret)
					return ret;
				continue;
			}

			if (WARN_ON(level >= RME_RTT_MAX_LEVEL))
				return -EINVAL;

			/* Recurse one level lower */
			ret = set_ipa_state(vcpu, ipa, ipa + map_size,
					    level + 1, ripas);
			if (ret)
				return ret;
		} else {
			WARN(1, "Unexpected error in %s: %#x\n", __func__,
			     ret);
			return -EINVAL;
		}
		ipa += map_size;
	}

	return 0;
}

static int realm_init_ipa_state(struct realm *realm,
				unsigned long ipa,
				unsigned long end,
				int level)
{
	unsigned long map_size = rme_rtt_level_mapsize(level);
	phys_addr_t rd_phys = virt_to_phys(realm->rd);
	int ret;

	while (ipa < end) {
		ret = rmi_rtt_init_ripas(rd_phys, ipa, level);

		if (RMI_RETURN_STATUS(ret) == RMI_ERROR_RTT) {
			int cur_level = RMI_RETURN_INDEX(ret);

			if (cur_level < level) {
				ret = realm_create_rtt_levels(realm, ipa,
							      cur_level,
							      level, NULL);
				if (ret)
					return ret;
				/* Retry with the RTT levels in place */
				continue;
			}

			/* There's an entry at a lower level, recurse */
			if (WARN_ON(level >= RME_RTT_MAX_LEVEL))
				return -EINVAL;

			realm_init_ipa_state(realm, ipa, ipa + map_size,
					     level + 1);
		} else if (WARN_ON(ret)) {
			return -ENXIO;
		}

		ipa += map_size;
	}

	return 0;
}

static int find_map_level(struct kvm *kvm, unsigned long start, unsigned long end)
{
	int level = RME_RTT_MAX_LEVEL;

	while (level > get_start_level(kvm) + 1) {
		unsigned long map_size = rme_rtt_level_mapsize(level - 1);

		if (!IS_ALIGNED(start, map_size) ||
		    (start + map_size) > end)
			break;

		level--;
	}

	return level;
}

int realm_set_ipa_state(struct kvm_vcpu *vcpu,
			unsigned long addr, unsigned long end,
			unsigned long ripas)
{
	int ret = 0;

	while (addr < end) {
		int level = find_map_level(vcpu->kvm, addr, end);
		unsigned long map_size = rme_rtt_level_mapsize(level);

		ret = set_ipa_state(vcpu, addr, addr + map_size, level, ripas);
		if (ret)
			break;

		addr += map_size;
	}

	return ret;
}

static int kvm_init_ipa_range_realm(struct kvm *kvm,
				    struct kvm_cap_arm_rme_init_ipa_args *args)
{
	int ret = 0;
	gpa_t addr, end;
	struct realm *realm = &kvm->arch.realm;

	addr = args->init_ipa_base;
	end = addr + args->init_ipa_size;

	if (end < addr)
		return -EINVAL;

	if (kvm_realm_state(kvm) != REALM_STATE_NEW)
		return -EBUSY;

	while (addr < end) {
		int level = find_map_level(kvm, addr, end);
		unsigned long map_size = rme_rtt_level_mapsize(level);

		ret = realm_init_ipa_state(realm, addr, addr + map_size, level);
		if (ret)
			break;

		addr += map_size;
	}

	return ret;
}

static int kvm_activate_realm(struct kvm *kvm)
{
	struct realm *realm = &kvm->arch.realm;

	if (kvm_realm_state(kvm) != REALM_STATE_NEW)
		return -EBUSY;

	if (rmi_realm_activate(virt_to_phys(realm->rd)))
		return -ENXIO;

	WRITE_ONCE(realm->state, REALM_STATE_ACTIVE);
	return 0;
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

	if (!kvm_rme_supports_sve())
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
	case KVM_CAP_ARM_RME_INIT_IPA_REALM: {
		struct kvm_cap_arm_rme_init_ipa_args args;
		void __user *argp = u64_to_user_ptr(cap->args[1]);

		if (copy_from_user(&args, argp, sizeof(args))) {
			r = -EFAULT;
			break;
		}

		r = kvm_init_ipa_range_realm(kvm, &args);
		break;
	}
	case KVM_CAP_ARM_RME_POPULATE_REALM: {
		struct kvm_cap_arm_rme_populate_realm_args args;
		void __user *argp = u64_to_user_ptr(cap->args[1]);

		if (copy_from_user(&args, argp, sizeof(args))) {
			r = -EFAULT;
			break;
		}

		r = kvm_populate_realm(kvm, &args);
		break;
	}
	case KVM_CAP_ARM_RME_ACTIVATE_REALM:
		r = kvm_activate_realm(kvm);
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

int kvm_rec_enter(struct kvm_vcpu *vcpu)
{
	struct rec *rec = &vcpu->arch.rec;

	if (kvm_realm_state(vcpu->kvm) != REALM_STATE_ACTIVE)
		return -EINVAL;

	return rmi_rec_enter(virt_to_phys(rec->rec_page),
			     virt_to_phys(rec->run));
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

	static_branch_enable(&kvm_rme_is_available);

	return 0;
}
