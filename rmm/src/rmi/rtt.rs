extern crate alloc;

use crate::event::RmiHandle;
use crate::granule::{
    is_granule_aligned, is_not_in_realm, set_granule, GranuleState, GRANULE_SIZE,
};
use crate::host;
use crate::host::DataPage;
use crate::listen;
use crate::measurement::HashContext;
use crate::realm::mm::rtt;
use crate::realm::mm::rtt::{RTT_MIN_BLOCK_LEVEL, RTT_PAGE_LEVEL};
use crate::realm::mm::stage2_tte::{level_mask, S2TTE};
use crate::realm::rd::{Rd, State};
use crate::rec::Rec;
use crate::rmi;
use crate::rmi::error::Error;
#[cfg(not(feature = "gst_page_table"))]
use crate::{get_granule, get_granule_if};
#[cfg(feature = "gst_page_table")]
use crate::{get_granule, get_granule_if, set_state_and_get_granule};

fn is_valid_rtt_cmd(rd: &Rd, ipa: usize, level: usize) -> bool {
    if level > RTT_PAGE_LEVEL {
        return false;
    }

    if ipa >= rd.ipa_size() {
        return false;
    }
    let mask = level_mask(level).unwrap_or(0);
    if ipa & mask as usize != ipa {
        return false;
    }
    true
}

pub fn set_event_handler(rmi: &mut RmiHandle) {
    listen!(rmi, rmi::RTT_CREATE, |arg, _ret, rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rtt_addr = arg[1];
        let rd = rd_granule.content::<Rd>()?;
        let ipa = arg[2];
        let level = arg[3];

        let min_level = rd.s2_starting_level() as usize + 1;

        if (level < min_level) || (level > RTT_PAGE_LEVEL) || !is_valid_rtt_cmd(&rd, ipa, level - 1)
        {
            return Err(Error::RmiErrorInput);
        }
        if rtt_addr == arg[0] {
            return Err(Error::RmiErrorInput);
        }
        let mut rtt_granule = get_granule_if!(rtt_addr, GranuleState::Delegated)?;

        // The below is added to avoid a fault regarding the RTT entry
        // during the `create_pgtbl_at()` in `rtt::create()`.
        #[cfg(not(kani))]
        rmm.page_table.map(rtt_addr, true);
        rtt::create(&rd, rtt_addr, ipa, level)?;
        set_granule(&mut rtt_granule, GranuleState::RTT)?;
        Ok(())
    });

    listen!(rmi, rmi::RTT_DESTROY, |arg, ret, _rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let ipa = arg[1];
        let level = arg[2];

        let min_level = rd.s2_starting_level() as usize + 1;

        if (level < min_level) || (level > RTT_PAGE_LEVEL) || !is_valid_rtt_cmd(&rd, ipa, level - 1)
        {
            return Err(Error::RmiErrorInput);
        }
        let (ipa, walk_top) = rtt::destroy(&rd, ipa, level, |t| {
            ret[2] = t;
        })?;
        ret[1] = ipa;
        ret[2] = walk_top;
        Ok(())
    });

    listen!(rmi, rmi::RTT_INIT_RIPAS, |arg, ret, _rmm| {
        let mut rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;
        let base = arg[1];
        let top = arg[2];

        if rd.state() != State::New {
            return Err(Error::RmiErrorRealm(0));
        }

        if top <= base {
            return Err(Error::RmiErrorInput);
        }

        if !is_valid_rtt_cmd(&rd, base, RTT_PAGE_LEVEL)
            || !is_valid_rtt_cmd(&rd, top, RTT_PAGE_LEVEL)
            || !rd.addr_in_par(base)
            || !rd.addr_in_par(top - GRANULE_SIZE)
        {
            return Err(Error::RmiErrorInput);
        }

        let out_top = rtt::init_ripas(&mut rd, base, top)?;
        ret[1] = out_top; //This is walk_top

        Ok(())
    });

    listen!(rmi, rmi::RTT_SET_RIPAS, |arg, ret, _rmm| {
        let base = arg[2];
        let top = arg[3];

        if arg[0] == arg[1] {
            warn!("Granules of RD and REC shouldn't be identical");
            return Err(Error::RmiErrorInput);
        }
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let mut rec_granule = get_granule_if!(arg[1], GranuleState::Rec)?;
        let mut rec = rec_granule.content_mut::<Rec<'_>>()?;
        if rec.realmid()? != rd.id() {
            warn!("RD:{:X} doesn't own REC:{:X}", arg[0], arg[1]);
            return Err(Error::RmiErrorRec);
        }

        if rec.ripas_addr() != base as u64 || rec.ripas_end() < top as u64 {
            return Err(Error::RmiErrorInput);
        }

        if !is_granule_aligned(base)
            || !is_granule_aligned(top)
            || !rd.addr_in_par(base)
            || top.checked_sub(GRANULE_SIZE).is_none()
            || !rd.addr_in_par(top - GRANULE_SIZE)
        {
            return Err(Error::RmiErrorInput);
        }

        let out_top = rtt::set_ripas(&rd, base, top, rec.ripas_state(), rec.ripas_flags())?;
        ret[1] = out_top;
        rec.set_ripas_addr(out_top as u64);
        Ok(())
    });

    listen!(rmi, rmi::RTT_READ_ENTRY, |arg, ret, _rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let ipa = arg[1];
        let level = arg[2];
        if !is_valid_rtt_cmd(&rd, ipa, level) {
            return Err(Error::RmiErrorInput);
        }

        let res = rtt::read_entry(&rd, ipa, level)?;
        ret[1..5].copy_from_slice(&res[0..4]);

        Ok(())
    });

    listen!(rmi, rmi::DATA_CREATE, |arg, _ret, rmm| {
        // target_pa: location where realm data is created.
        let rd = arg[0];
        let target_pa = arg[1];
        let ipa = arg[2];
        let src_pa = arg[3];
        let flags = arg[4];

        if target_pa == rd || target_pa == src_pa || rd == src_pa {
            return Err(Error::RmiErrorInput);
        }

        // rd granule lock
        let mut rd_granule = get_granule_if!(rd, GranuleState::RD)?;
        let mut rd = rd_granule.content_mut::<Rd>()?;

        // Make sure DATA_CREATE is only processed
        // when the realm is in its New state.
        if !rd.at_state(State::New) {
            return Err(Error::RmiErrorRealm(0));
        }

        validate_ipa(&rd, ipa)?;

        if !is_not_in_realm(src_pa) {
            return Err(Error::RmiErrorInput);
        };

        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        let mut target_page = target_page_granule.content_mut::<DataPage>()?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(target_pa, true);

        // copy src to target
        #[cfg(not(kani))]
        rmm.page_table.map(src_pa, false);
        host::copy_to_obj::<DataPage>(src_pa, &mut target_page).ok_or(Error::RmiErrorInput)?;
        #[cfg(not(kani))]
        rmm.page_table.unmap(src_pa);

        // map ipa to taget_pa in S2 table
        rtt::data_create(&rd, ipa, target_pa, false)?;

        #[cfg(not(kani))]
        // `rsi` is currently not reachable in model checking harnesses
        HashContext::new(&mut rd)?.measure_data_granule(&target_page, ipa, flags)?;

        set_granule(&mut target_page_granule, GranuleState::Data)?;
        Ok(())
    });

    listen!(rmi, rmi::DATA_CREATE_UNKNOWN, |arg, _ret, rmm| {
        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;

        // target_phys: location where realm data is created.
        let target_pa = arg[1];
        let ipa = arg[2];
        if target_pa == arg[0] {
            return Err(Error::RmiErrorInput);
        }

        validate_ipa(&rd, ipa)?;

        // 0. Make sure granule state can make a transition to DATA
        // data granule lock for the target page
        let mut target_page_granule = get_granule_if!(target_pa, GranuleState::Delegated)?;
        #[cfg(not(kani))]
        // `page_table` is currently not reachable in model checking harnesses
        rmm.page_table.map(target_pa, true);

        // 1. map ipa to target_pa in S2 table
        rtt::data_create(&rd, ipa, target_pa, true)?;

        // TODO: 2. perform measure
        // L0czek - not needed here see: tf-rmm/runtime/rmi/rtt.c:883
        set_granule(&mut target_page_granule, GranuleState::Data)?;
        Ok(())
    });

    listen!(rmi, rmi::DATA_DESTROY, |arg, ret, _rmm| {
        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let ipa = arg[1];

        if !rd.addr_in_par(ipa) || !is_valid_rtt_cmd(&rd, ipa, RTT_PAGE_LEVEL) {
            return Err(Error::RmiErrorInput);
        }

        let (pa, top) = rtt::data_destroy(&rd, ipa, |t| {
            ret[2] = t;
        })?;

        // data granule lock and change state
        #[cfg(feature = "gst_page_table")]
        set_state_and_get_granule!(pa, GranuleState::Delegated)?;

        #[cfg(not(feature = "gst_page_table"))]
        {
            let mut granule = get_granule!(pa)?;
            set_granule(&mut granule, GranuleState::Delegated)?;
        }

        ret[1] = pa;
        ret[2] = top;
        Ok(())
    });

    // Map an unprotected IPA to a non-secure PA.
    listen!(rmi, rmi::RTT_MAP_UNPROTECTED, |arg, _ret, _rmm| {
        let ipa = arg[1];
        let level = arg[2];
        let host_s2tte = arg[3];
        let s2tte = S2TTE::from(host_s2tte);
        if !s2tte.is_host_ns_valid(level) {
            return Err(Error::RmiErrorInput);
        }

        // rd granule lock
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;

        if !is_valid_rtt_cmd(&rd, ipa, level) {
            return Err(Error::RmiErrorInput);
        }
        rtt::map_unprotected(&rd, ipa, level, host_s2tte)?;
        Ok(())
    });

    // Unmap a non-secure PA at an unprotected IPA
    listen!(rmi, rmi::RTT_UNMAP_UNPROTECTED, |arg, ret, _rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;

        let ipa = arg[1];

        let level = arg[2];
        if (level < RTT_MIN_BLOCK_LEVEL)
            || (level > RTT_PAGE_LEVEL)
            || !is_valid_rtt_cmd(&rd, ipa, level)
        {
            return Err(Error::RmiErrorInput);
        }

        let top = rtt::unmap_unprotected(&rd, ipa, level, |t| {
            ret[1] = t;
        })?;
        ret[1] = top;

        Ok(())
    });

    // Destroy a homogeneous RTT and map as a bigger block at its parent RTT
    listen!(rmi, rmi::RTT_FOLD, |arg, ret, _rmm| {
        let rd_granule = get_granule_if!(arg[0], GranuleState::RD)?;
        let rd = rd_granule.content::<Rd>()?;
        let ipa = arg[1];
        let level = arg[2];

        let min_level = rd.s2_starting_level() as usize + 1;

        if (level < min_level) || (level > RTT_PAGE_LEVEL) || !is_valid_rtt_cmd(&rd, ipa, level - 1)
        {
            return Err(Error::RmiErrorInput);
        }

        let rtt = rtt::fold(&rd, ipa, level)?;
        ret[1] = rtt;
        Ok(())
    });
}

pub fn validate_ipa(rd: &Rd, ipa: usize) -> Result<(), Error> {
    if !is_granule_aligned(ipa) {
        error!("ipa: {:x} is not aligned with {:x}", ipa, GRANULE_SIZE);
        return Err(Error::RmiErrorInput);
    }

    if !rd.addr_in_par(ipa) {
        error!(
            "ipa: {:x} is not in protected ipa range {:x}",
            ipa,
            rd.par_size()
        );
        return Err(Error::RmiErrorInput);
    }

    if !is_valid_rtt_cmd(rd, ipa, RTT_PAGE_LEVEL) {
        return Err(Error::RmiErrorInput);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::granule::GRANULE_SIZE;
    use crate::realm::rd::{Rd, State};
    use crate::rmi::*;
    use crate::test_utils::{mock, *};

    use alloc::vec;

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_rtt_create
    // Covered RMIs: RTT_CREATE, RTT_DESTROY, RTT_READ_ENTRY
    #[test]
    fn rmi_rtt_create_positive() {
        let rd = realm_create();

        let (rtt1, rtt2, rtt3, rtt4) = (
            mock::host::alloc_granule(IDX_RTT_LEVEL1),
            mock::host::alloc_granule(IDX_RTT_LEVEL2),
            mock::host::alloc_granule(IDX_RTT_LEVEL3),
            mock::host::alloc_granule(IDX_RTT_OTHER),
        );

        for rtt in &[rtt1, rtt2, rtt3, rtt4] {
            let ret = rmi::<GRANULE_DELEGATE>(&[*rtt]);
            assert_eq!(ret[0], SUCCESS);
        }

        let test_data = vec![
            (rtt1, 0x0, 0x1),
            (rtt2, 0x0, 0x2),
            (rtt3, 0x0, 0x3),
            (rtt4, 0x40000000, 0x2),
        ];

        unsafe {
            let rd_obj = &*(rd as *const Rd);
            assert!(rd_obj.at_state(State::New));
        };

        for (rtt, ipa, level) in &test_data {
            let ret = rmi::<RTT_CREATE>(&[rd, *rtt, *ipa, *level]);
            assert_eq!(ret[0], SUCCESS);
        }

        let (rtt4_ipa, rtt4_level) = (test_data[3].1, test_data[3].2);
        let ret = rmi::<RTT_READ_ENTRY>(&[rd, rtt4_ipa, rtt4_level - 1]);
        assert_eq!(ret[0], SUCCESS);

        let (state, desc) = (ret[2], ret[3]);
        const RMI_TABLE: usize = 2;
        assert_eq!(state, RMI_TABLE);
        assert_eq!(desc, rtt4);

        for (_, ipa, level) in test_data.iter().rev() {
            let ret = rmi::<RTT_DESTROY>(&[rd, *ipa, *level]);
            assert_eq!(ret[0], SUCCESS);
        }

        for rtt in &[rtt1, rtt2, rtt3, rtt4] {
            let ret = rmi::<GRANULE_UNDELEGATE>(&[*rtt]);
            assert_eq!(ret[0], SUCCESS);
        }

        realm_destroy(rd);
    }

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_rtt_init_ripas
    // Covered RMIs: RTT_INIT_RIPAS, RTT_READ_ENTRY
    #[test]
    fn rmi_rtt_init_ripas_positive() {
        let rd = realm_create();
        let ipa = 0;
        mock::host::map(rd, ipa);

        let base = (ipa / L3_SIZE) * L3_SIZE;
        let top = base + L3_SIZE;
        let ret = rmi::<RTT_INIT_RIPAS>(&[rd, base, top]);
        assert_eq!(ret[0], SUCCESS);
        assert_eq!(ret[1], top);

        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[0], SUCCESS);

        let (level, ripas) = (ret[1], ret[4]);
        const RMI_RAM: usize = 1;
        assert_eq!(level, MAP_LEVEL);
        assert_eq!(ripas, RMI_RAM);

        mock::host::unmap(rd, ipa, false);

        realm_destroy(rd);
    }

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_rtt_map_unprotected
    // Covered RMIs: RTT_MAP_UNPROTECTED, RTT_UNMAP_UNPROTECTED
    #[test]
    fn rmi_rtt_map_unprotected_positive() {
        let rd = realm_create();

        const IPA_ADDR_UNPROTECTED_UNASSIGNED: usize = (1 << (IPA_WIDTH - 1)) + L3_SIZE;
        mock::host::map(rd, IPA_ADDR_UNPROTECTED_UNASSIGNED);

        let ipa = IPA_ADDR_UNPROTECTED_UNASSIGNED;
        let level = MAP_LEVEL;
        let ns = mock::host::alloc_granule(IDX_NS_DESC);
        let desc = ns | ATTR_NORMAL_WB_WA_RA | ATTR_STAGE2_AP_RW | ATTR_INNER_SHARED;

        let ret = rmi::<RTT_MAP_UNPROTECTED>(&[rd, ipa, level, desc]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, level]);
        assert_eq!(ret[0], SUCCESS);

        let (_level, state, out_desc, _ripas) = (ret[1], ret[2], ret[3], ret[4]);
        const RMI_ASSIGNED: usize = 1;
        assert_eq!(state, RMI_ASSIGNED);
        assert_eq!(out_desc, desc);

        let ret = rmi::<RTT_UNMAP_UNPROTECTED>(&[rd, ipa, level]);
        assert_eq!(ret[0], SUCCESS);

        mock::host::unmap(rd, ipa, false);

        realm_destroy(rd);
    }

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_data_destroy
    // Covered RMIs: DATA_CREATE, DATA_CREATE_UNKNOWN, DATA_DESTROY
    #[test]
    fn rmi_data_create_positive() {
        let rd = realm_create();

        const IPA_ADDR_ASSIGNED: usize = GRANULE_SIZE;
        const IPA_ADDR_DATA: usize = GRANULE_SIZE * 3;
        const IPA_ADDR_PROTECTED_ASSIGNED_EMPTY: usize = GRANULE_SIZE * 4;

        const RMI_UNASSIGNED: usize = 0;
        const RMI_DESTROYED: usize = 2;
        const RMI_EMPTY: usize = 0;

        data_create(rd, IPA_ADDR_ASSIGNED, IDX_DATA1, IDX_SRC1);

        data_create(rd, IPA_ADDR_DATA, IDX_DATA2, IDX_SRC2);

        let ipa = IPA_ADDR_ASSIGNED;
        let ret = rmi::<DATA_DESTROY>(&[rd, ipa]);
        assert_eq!(ret[0], SUCCESS);

        let (data, top) = (ret[1], ret[2]);
        assert_eq!(data, granule_addr(IDX_DATA1));
        assert_eq!(top, IPA_ADDR_DATA);

        // Check for RIPAS and HIPAS from RIPAS = RAM
        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[0], SUCCESS);

        let (_level, state, _desc, ripas) = (ret[1], ret[2], ret[3], ret[4]);
        assert_eq!(state, RMI_UNASSIGNED);
        assert_eq!(ripas, RMI_DESTROYED);

        // Check for RIPAS transition from ASSIGNED,DESTROYED
        let data = mock::host::alloc_granule(IDX_DATA3);
        let ret = rmi::<GRANULE_DELEGATE>(&[data]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<DATA_CREATE_UNKNOWN>(&[rd, data, ipa]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<DATA_DESTROY>(&[rd, ipa]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[0], SUCCESS);

        let (_level, state, _desc, ripas) = (ret[1], ret[2], ret[3], ret[4]);
        assert_eq!(state, RMI_UNASSIGNED);
        assert_eq!(ripas, RMI_DESTROYED);

        // Check for RIPAS and HIPAS from RIPAS = EMPTY
        let ipa = IPA_ADDR_PROTECTED_ASSIGNED_EMPTY;
        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[0], SUCCESS);

        mock::host::map(rd, ipa);

        let data = mock::host::alloc_granule(IDX_DATA4);
        let ret = rmi::<GRANULE_DELEGATE>(&[data]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<DATA_CREATE_UNKNOWN>(&[rd, data, ipa]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<DATA_DESTROY>(&[rd, ipa]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<RTT_READ_ENTRY>(&[rd, ipa, MAP_LEVEL]);
        assert_eq!(ret[0], SUCCESS);

        let (_level, state, _desc, ripas) = (ret[1], ret[2], ret[3], ret[4]);
        assert_eq!(state, RMI_UNASSIGNED);
        assert_eq!(ripas, RMI_EMPTY);

        let ret = rmi::<DATA_DESTROY>(&[rd, IPA_ADDR_DATA]);
        assert_eq!(ret[0], SUCCESS);

        // Cleanup
        mock::host::unmap(rd, ipa, false);
        for idx in IDX_DATA1..IDX_DATA4 + 1 {
            let ret = rmi::<GRANULE_UNDELEGATE>(&[granule_addr(idx)]);
            assert_eq!(ret[0], SUCCESS);
        }

        realm_destroy(rd);
    }

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_rtt_fold
    // Covered RMIs: RTT_FOLD
    #[test]
    fn rmi_rtt_fold_positive() {
        let rd = realm_create();

        const IPA_HOMOGENEOUS_RTT: usize = 0;
        let ipa = IPA_HOMOGENEOUS_RTT;

        mock::host::map(rd, ipa);

        let base = ipa;
        let top = ipa + L2_SIZE;
        let ret = rmi::<RTT_INIT_RIPAS>(&[rd, base, top]);
        assert_eq!(ret[0], SUCCESS);

        // Save Parent rtte.addr for comparision
        let ret = rmi::<RTT_READ_ENTRY>(&[rd, base, MAP_LEVEL - 1]);
        let (_level, _state, parent_desc, _ripas) = (ret[1], ret[2], ret[3], ret[4]);
        assert_eq!(ret[0], SUCCESS);

        // Save fold.addr, fold.ripas for Positive Observability
        let ret = rmi::<RTT_READ_ENTRY>(&[rd, base, MAP_LEVEL]);
        let (_level, fold_state, fold_desc, fold_ripas) = (ret[1], ret[2], ret[3], ret[4]);
        assert_eq!(ret[0], SUCCESS);

        let ret = rmi::<RTT_FOLD>(&[rd, base, MAP_LEVEL]);
        assert_eq!(ret[0], SUCCESS);
        let out_rtt = ret[1];
        assert_eq!(out_rtt, parent_desc);

        // Compare rtte_addr, rtte_ripas in folded RTTE
        let ret = rmi::<RTT_READ_ENTRY>(&[rd, base, MAP_LEVEL - 1]);
        assert_eq!(ret[0], SUCCESS);

        // Compare HIPAS, RIPAS and addr of parent RTTE to its child RTTE
        let (_level, state, desc, ripas) = (ret[1], ret[2], ret[3], ret[4]);
        assert_eq!(fold_state, state);
        assert_eq!(fold_desc, desc);
        assert_eq!(fold_ripas, ripas);

        mock::host::unmap(rd, ipa, true);

        realm_destroy(rd);
    }

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_rtt_set_ripas
    // Covered RMIs: RTT_SET_RIPAS
    #[test]
    fn rmi_rtt_set_ripas_positive() {
        use crate::rmi::rec::run::Run;
        use crate::rsi::PSCI_CPU_ON;

        let rd = mock::host::realm_setup();

        let (rec1, run1) = (granule_addr(IDX_REC1), granule_addr(IDX_REC1_RUN));
        let ret = rmi::<REC_ENTER>(&[rec1, run1]);
        assert_eq!(ret[0], SUCCESS);

        let ipa = 0;
        mock::host::map(rd, ipa);

        unsafe {
            let run = &*(run1 as *const Run);
            let (base, top) = run.ripas();

            let ret = rmi::<RTT_SET_RIPAS>(&[rd, rec1, base as usize, top as usize]);
            assert_eq!(ret[0], SUCCESS);
        }

        mock::host::unmap(rd, ipa, false);
        mock::host::realm_teardown(rd);
    }
}
