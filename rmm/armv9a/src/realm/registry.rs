use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::realm::mm::IPATranslation;
use monitor::realm::vcpu::VCPU;
use monitor::realm::Realm;
use monitor::rmi::rec::run::{Run, REC_ENTRY_FLAG_EMUL_MMIO};
use monitor::rmi::MapProt;

use crate::gic;
use crate::gic::{GIC_FEATURES, ICH_HCR_EL2_EOI_COUNT_MASK, ICH_HCR_EL2_NS_MASK};
use crate::helper;
use crate::helper::bits_in_reg;
use crate::helper::VTTBR_EL2;
use crate::helper::{EsrEl2, ESR_EL2_EC_DATA_ABORT};
use crate::realm;
use crate::realm::context::Context;
use crate::realm::mm::page_table::pte;
use crate::realm::mm::stage2_translation::Stage2Translation;
use crate::realm::mm::stage2_tte::S2TTE;
use crate::realm::mm::stage2_tte::{desc_type, invalid_hipas, invalid_ripas};
use crate::realm::mm::translation_granule_4k::RawPTE;
use crate::realm::timer;
use monitor::realm::config::RealmConfig;
use monitor::rmi::error::Error;
use monitor::rmi::error::InternalError::*;
use monitor::rmi::rtt_entry_state;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::mutex::Mutex;
use spinning_top::Spinlock;

type RealmMutex = Arc<Mutex<Realm<Context>>>;
type RealmMap = BTreeMap<usize, RealmMutex>;
static RMS: Spinlock<(usize, RealmMap)> = Spinlock::new((0, BTreeMap::new()));

fn enter() -> [usize; 4] {
    unsafe {
        if let Some(vcpu) = realm::vcpu::current() {
            if vcpu.is_realm_dead() {
                vcpu.from_current();
            } else {
                vcpu.realm.lock().page_table.lock().clean();
                return helper::rmm_exit([0; 4]);
            }
        }
        [0, 0, 0, 0]
    }
}

fn exit() {
    unsafe {
        if let Some(vcpu) = realm::vcpu::current() {
            vcpu.from_current();
        }
    }
}

fn get_realm(id: usize) -> Option<RealmMutex> {
    RMS.lock().1.get(&id).map(|realm| Arc::clone(realm))
}

#[derive(Debug)]
pub struct RMI;
impl RMI {
    pub fn new() -> &'static RMI {
        &RMI {}
    }
}

impl monitor::rmi::Interface for RMI {
    fn create_realm(&self, vmid: u16) -> Result<usize, Error> {
        let mut rms = RMS.lock();

        for (_, realm) in &rms.1 {
            if vmid == realm.lock().vmid {
                return Err(Error::RmiErrorInput);
            }
        }

        let id = rms.0;
        let s2_table = Arc::new(Mutex::new(
            Box::new(Stage2Translation::new()) as Box<dyn IPATranslation>
        ));
        let realm = Realm::new(id, vmid, s2_table);

        rms.0 += 1;
        rms.1.insert(id, realm.clone());

        Ok(id)
    }

    fn create_vcpu(&self, id: usize) -> Result<usize, Error> {
        let realm = get_realm(id).ok_or(Error::RmiErrorInput)?;

        let page_table = realm.lock().page_table.lock().get_base_address();
        let vttbr = bits_in_reg(VTTBR_EL2::VMID, id as u64)
            | bits_in_reg(VTTBR_EL2::BADDR, page_table as u64);

        let vcpu = VCPU::new(realm.clone());
        vcpu.lock().context.sys_regs.vttbr = vttbr;
        timer::init_timer(&mut vcpu.lock());
        gic::init_gic(&mut vcpu.lock());

        realm.lock().vcpus.push(vcpu);
        let vcpuid = realm.lock().vcpus.len() - 1;
        Ok(vcpuid)
    }

    fn remove(&self, id: usize) -> Result<(), Error> {
        RMS.lock().1.remove(&id).ok_or(Error::RmiErrorInput)?;
        Ok(())
    }

    fn run(&self, id: usize, vcpu: usize, incr_pc: usize) -> Result<[usize; 4], Error> {
        if incr_pc == 1 {
            get_realm(id)
                .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                .lock()
                .context
                .elr += 4;
        }
        debug!(
            "resuming: {:#x}",
            get_realm(id)
                .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                .lock()
                .context
                .elr
        );

        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .vcpus
            .get(vcpu)
            .map(|vcpu| VCPU::into_current(&mut *vcpu.lock()));

        trace!("Switched to VCPU {} on Realm {}", vcpu, id);
        let ret = enter();

        exit();
        Ok(ret)
    }

    fn map(
        &self,
        id: usize,
        guest: usize,
        phys: usize,
        size: usize,
        prot: usize,
    ) -> Result<(), Error> {
        let mut flags = 0;
        let prot = MapProt::new(prot);

        if prot.is_set(MapProt::NS_PAS) {
            flags |= helper::bits_in_reg(RawPTE::NS, 0b1);
        }

        // TODO:  define bit mask
        flags |= helper::bits_in_reg(RawPTE::S2AP, pte::permission::RW);
        if prot.is_set(MapProt::DEVICE) {
            flags |= helper::bits_in_reg(RawPTE::ATTR, pte::attribute::DEVICE_NGNRE);
            flags |= helper::bits_in_reg(RawPTE::NS, 0b1);
        } else {
            flags |= helper::bits_in_reg(RawPTE::ATTR, pte::attribute::NORMAL);
        }

        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .set_pages(
                GuestPhysAddr::from(guest),
                PhysAddr::from(phys),
                size,
                flags as usize,
                false,
            )?;

        Ok(())
    }

    fn unmap(&self, id: usize, guest: usize, size: usize) -> Result<usize, Error> {
        let pa = get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .ipa_to_pa(GuestPhysAddr::from(guest), 3)
            .ok_or(Error::RmiErrorInput)?;

        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .unset_pages(GuestPhysAddr::from(guest), size);

        //TODO change GPT to nonsecure
        //TODO zeroize memory
        Ok(pa.into())
    }

    fn set_reg(&self, id: usize, vcpu: usize, register: usize, value: usize) -> Result<(), Error> {
        match register {
            0..=30 => {
                get_realm(id)
                    .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                    .lock()
                    .context
                    .gp_regs[register] = value as u64;
                Ok(())
            }
            31 => {
                get_realm(id)
                    .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                    .lock()
                    .context
                    .elr = value as u64;
                Ok(())
            }
            32 => {
                get_realm(id)
                    .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                    .lock()
                    .context
                    .spsr = value as u64;
                Ok(())
            }
            _ => Err(Error::RmiErrorInput),
        }?;
        Ok(())
    }

    fn get_reg(&self, id: usize, vcpu: usize, register: usize) -> Result<usize, Error> {
        match register {
            0..=30 => {
                let value = get_realm(id)
                    .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                    .lock()
                    .context
                    .gp_regs[register];
                Ok(value as usize)
            }
            31 => {
                let value = get_realm(id)
                    .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or(Error::RmiErrorOthers(NotExistVCPU))?
                    .lock()
                    .context
                    .elr;
                Ok(value as usize)
            }
            _ => Err(Error::RmiErrorInput),
        }
    }

    fn receive_gic_state_from_host(&self, id: usize, vcpu: usize, run: &Run) -> Result<(), Error> {
        let realm = get_realm(id).ok_or(Error::RmiErrorOthers(NotExistRealm))?;
        let locked_realm = realm.lock();
        let vcpu = locked_realm
            .vcpus
            .get(vcpu)
            .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
        let gic_state = &mut vcpu.lock().context.gic_state;
        let nr_lrs = GIC_FEATURES.nr_lrs;

        gic_state.ich_lr_el2[..nr_lrs].copy_from_slice(&unsafe { run.entry_gic_lrs() }[..nr_lrs]);
        gic_state.ich_hcr_el2 &= !ICH_HCR_EL2_NS_MASK;
        gic_state.ich_hcr_el2 |= (unsafe { run.entry_gic_hcr() } & ICH_HCR_EL2_NS_MASK);
        Ok(())
    }

    fn send_gic_state_to_host(&self, id: usize, vcpu: usize, run: &mut Run) -> Result<(), Error> {
        let realm = get_realm(id).ok_or(Error::RmiErrorOthers(NotExistRealm))?;
        let mut locked_realm = realm.lock();
        let vcpu = locked_realm
            .vcpus
            .get_mut(vcpu)
            .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
        let gic_state = &mut vcpu.lock().context.gic_state;
        let nr_lrs = GIC_FEATURES.nr_lrs;

        (&mut unsafe { run.exit_gic_lrs_mut() }[..nr_lrs])
            .copy_from_slice(&gic_state.ich_lr_el2[..nr_lrs]);
        unsafe {
            run.set_gic_misr(gic_state.ich_misr_el2);
            run.set_gic_vmcr(gic_state.ich_vmcr_el2);
            run.set_gic_hcr(
                gic_state.ich_hcr_el2 & (ICH_HCR_EL2_EOI_COUNT_MASK | ICH_HCR_EL2_NS_MASK),
            );
        }
        Ok(())
    }

    fn emulate_mmio(&self, id: usize, vcpu: usize, run: &Run) -> Result<(), Error> {
        let realm = get_realm(id).ok_or(Error::RmiErrorOthers(NotExistRealm))?;
        let mut locked_realm = realm.lock();
        let vcpu = locked_realm
            .vcpus
            .get_mut(vcpu)
            .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
        let context = &mut vcpu.lock().context;

        let flags = unsafe { run.entry_flags() };

        // Host has not completed emulation for an Emulatable Abort.
        if (flags & REC_ENTRY_FLAG_EMUL_MMIO) == 0 {
            return Ok(());
        }

        let esr_el2 = context.sys_regs.esr_el2;
        let esr = EsrEl2::new(esr_el2);
        let isv = esr.get_masked_value(EsrEl2::ISV);
        let ec = esr.get_masked_value(EsrEl2::EC);
        let wnr = esr.get_masked_value(EsrEl2::WNR);
        let rt = esr.get_masked_value(EsrEl2::SRT) as usize;

        if ec != ESR_EL2_EC_DATA_ABORT || isv == 0 {
            return Err(Error::RmiErrorRec);
        }

        // MMIO read case
        if wnr == 0 && rt != 31 {
            let sas = esr.get_masked_value(EsrEl2::SAS);
            let mask: u64 = match sas {
                0 => 0xff,                // byte
                1 => 0xffff,              // half-word
                2 => 0xffffffff,          // word
                3 => 0xffffffff_ffffffff, // double word
                _ => unreachable!(),      // SAS consists of two bits
            };
            let val = unsafe { run.entry_gpr0() } & mask;
            let sign_extended = esr.get_masked_value(EsrEl2::SSE);
            if sign_extended != 0 {
                // TODO
                unimplemented!();
            }
            context.gp_regs[rt] = val;
        }
        context.elr += 4;
        Ok(())
    }

    fn send_mmio_write(&self, id: usize, vcpu: usize, run: &mut Run) -> Result<(), Error> {
        let realm = get_realm(id).ok_or(Error::RmiErrorOthers(NotExistRealm))?;
        let mut locked_realm = realm.lock();
        let vcpu = locked_realm
            .vcpus
            .get_mut(vcpu)
            .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
        let context = &mut vcpu.lock().context;

        let esr_el2 = context.sys_regs.esr_el2;
        let esr = EsrEl2::new(esr_el2);
        let isv = esr.get_masked_value(EsrEl2::ISV);
        let ec = esr.get_masked_value(EsrEl2::EC);
        let wnr = esr.get_masked_value(EsrEl2::WNR);
        let rt = esr.get_masked_value(EsrEl2::SRT) as usize;

        // wnr == 1: caused by writing
        if ec != ESR_EL2_EC_DATA_ABORT || isv == 0 || wnr == 0 {
            return Ok(());
        }

        if rt == 31 {
            // xzr
            unsafe {
                run.set_gpr0(0);
            }
        } else {
            let sas = esr.get_masked_value(EsrEl2::SAS);
            let mask: u64 = match sas {
                0 => 0xff,                // byte
                1 => 0xffff,              // half-word
                2 => 0xffffffff,          // word
                3 => 0xffffffff_ffffffff, // double word
                _ => unreachable!(),      // SAS consists of two bits
            };
            let sign_extended = esr.get_masked_value(EsrEl2::SSE);
            if sign_extended != 0 {
                // TODO
                unimplemented!();
            }
            unsafe {
                run.set_gpr0(context.gp_regs[rt] & mask);
            }
        }
        Ok(())
    }

    fn send_timer_state_to_host(&self, id: usize, vcpu: usize, run: &mut Run) -> Result<(), Error> {
        let realm = get_realm(id).ok_or(Error::RmiErrorOthers(NotExistRealm))?;
        let mut locked_realm = realm.lock();
        let vcpu = locked_realm
            .vcpus
            .get_mut(vcpu)
            .ok_or(Error::RmiErrorOthers(NotExistVCPU))?;
        let timer = &vcpu.lock().context.timer;

        unsafe {
            run.set_cntv_ctl(timer.cntv_ctl_el0);
            run.set_cntv_cval(timer.cntv_cval_el0 - timer.cntvoff_el2);
            run.set_cntp_ctl(timer.cntp_ctl_el0);
            run.set_cntp_cval(timer.cntp_cval_el0 - timer.cntpoff_el2);
        }
        Ok(())
    }

    fn realm_config(&self, id: usize, config_ipa: usize) -> Result<(), Error> {
        let res = get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .ipa_to_pa(GuestPhysAddr::from(config_ipa), 3);
        if let Some(pa) = res {
            let pa: usize = pa.into();
            // TODO: Receive ipa width (== 33) from the Host's argument
            //       and store it inside REC as well
            unsafe { RealmConfig::init(pa, 33) };
            Ok(())
        } else {
            Err(Error::RmiErrorInput)
        }
    }

    fn rtt_read_entry(&self, id: usize, ipa: usize, level: usize) -> Result<[usize; 4], Error> {
        let (s2tte, last_level) = get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .ipa_to_pte(GuestPhysAddr::from(ipa), level)
            .map_or((0, level), |pte| pte);

        let r1 = last_level;
        let (mut r2, mut r3, mut r4) = (0, 0, 0);

        let s2tte = S2TTE::from(s2tte as usize);

        if s2tte.is_unassigned() {
            let ripas = s2tte.get_masked_value(S2TTE::INVALID_RIPAS);
            r2 = rtt_entry_state::RMI_UNASSIGNED;
            r4 = ripas as usize;
        } else if s2tte.is_destroyed() {
            r2 = rtt_entry_state::RMI_DESTROYED;
        } else if s2tte.is_assigned() {
            r2 = rtt_entry_state::RMI_ASSIGNED;
            r3 = s2tte.address(last_level).ok_or(Error::RmiErrorRtt)?.into();
            r4 = invalid_ripas::EMPTY as usize;
        } else if s2tte.is_valid(last_level, false) {
            r2 = rtt_entry_state::RMI_ASSIGNED;
            r3 = s2tte.address(last_level).ok_or(Error::RmiErrorRtt)?.into();
            r4 = invalid_ripas::RAM as usize;
        } else if s2tte.is_valid(last_level, true) {
            r2 = rtt_entry_state::RMI_VALID_NS;
            let addr_mask = match level {
                1 => S2TTE::ADDR_L1_PAGE,
                2 => S2TTE::ADDR_L2_PAGE,
                3 => S2TTE::ADDR_L3_PAGE,
                _ => {
                    return Err(Error::RmiErrorRtt);
                }
            };
            let mask = addr_mask | S2TTE::MEMATTR | S2TTE::AP | S2TTE::SH;
            r3 = (s2tte.get() & mask) as usize;
        } else if s2tte.is_table(last_level) {
            r2 = rtt_entry_state::RMI_TABLE;
            r3 = s2tte.address(3).ok_or(Error::RmiErrorRtt)?.into();
        } else {
            error!("Unexpected S2TTE value retrieved!");
        }
        Ok([r1, r2, r3, r4])
    }

    fn rtt_map_unprotected(
        &self,
        id: usize,
        ipa: usize,
        _level: usize,
        host_s2tte: usize,
    ) -> Result<(), Error> {
        let size = monitor::rmm::granule::GRANULE_SIZE;
        let mut new_s2tte = host_s2tte as u64;
        new_s2tte |= helper::bits_in_reg(S2TTE::NS, 1)
            | helper::bits_in_reg(S2TTE::XN, 1)
            | helper::bits_in_reg(S2TTE::AF, 1)
            | helper::bits_in_reg(S2TTE::DESC_TYPE, desc_type::L3_PAGE);
        let s2tte = S2TTE::from(new_s2tte as usize);
        let pa = s2tte.get_masked(S2TTE::ADDR_FULL);
        let flags = s2tte.get_masked(S2TTE::PAGE_FLAGS);
        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .set_pages(
                GuestPhysAddr::from(ipa),
                PhysAddr::from(pa),
                size,
                flags as usize,
                true,
            )?;

        Ok(())
    }

    fn make_shared(&self, id: usize, ipa: usize, level: usize) -> Result<(), Error> {
        let (s2tte, last_level) = get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .ipa_to_pte(GuestPhysAddr::from(ipa), level)
            .ok_or(Error::RmiErrorRtt)?;
        if level != last_level {
            return Err(Error::RmiErrorRtt);
        }

        let s2tte = S2TTE::from(s2tte as usize);

        // Reference (tf-rmm)      : smc_rtt_set_ripas() in runtime/rmi/rtt.c
        //           (realm-linux) : __set_memory_encrypted() in arch/arm64/mm/pageattr.c
        //           (nw-linux)    : set_ipa_state() and kvm_realm_unmap_range() in arch/arm64/kvm/rme.c
        //           (rmm-spec)    : Figure D2.1 Realm shared memory protocol flow
        if s2tte.is_valid(level, false) {
            // the case for ipa's range 0x8840_0000 - in realm-linux booting
            let pa: usize = s2tte.address(level).ok_or(Error::RmiErrorRtt)?.into();
            let size = monitor::rmm::granule::GRANULE_SIZE;
            let mut flags = 0;
            flags |= helper::bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::ASSIGNED);
            flags |= helper::bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);

            get_realm(id)
                .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                .lock()
                .page_table
                .lock()
                .set_pages(
                    GuestPhysAddr::from(ipa),
                    PhysAddr::from(pa),
                    size,
                    flags as usize,
                    true,
                )?;
        } else if s2tte.is_unassigned() || s2tte.is_assigned() {
            let pa: usize = s2tte.address(level).ok_or(Error::RmiErrorRtt)?.into();
            let size = monitor::rmm::granule::GRANULE_SIZE;
            let mut flags = 0;
            flags |= helper::bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
            get_realm(id)
                .ok_or(Error::RmiErrorOthers(NotExistRealm))?
                .lock()
                .page_table
                .lock()
                .set_pages(
                    GuestPhysAddr::from(ipa),
                    PhysAddr::from(pa),
                    size,
                    flags as usize,
                    true,
                )?;
        }

        Ok(())
    }

    fn data_destroy(&self, id: usize, ipa: usize) -> Result<usize, Error> {
        let (s2tte, last_level) = get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .ipa_to_pte(GuestPhysAddr::from(ipa), 3)
            .ok_or(Error::RmiErrorRtt)?;

        if last_level != 3 {
            return Err(Error::RmiErrorRtt);
        }

        let s2tte = S2TTE::from(s2tte as usize);

        let valid = s2tte.is_valid(last_level, false);
        if !valid && !s2tte.is_assigned() {
            return Err(Error::RmiErrorRtt);
        }

        let pa = s2tte.address(last_level).ok_or(Error::RmiErrorRtt)?.into();

        let mut flags = 0 as u64;
        if valid {
            flags |= helper::bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::DESTROYED);
        } else {
            flags |= helper::bits_in_reg(S2TTE::INVALID_HIPAS, invalid_hipas::UNASSIGNED);
            flags |= helper::bits_in_reg(S2TTE::INVALID_RIPAS, invalid_ripas::EMPTY);
        }
        let size = monitor::rmm::granule::GRANULE_SIZE;
        get_realm(id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
            .lock()
            .set_pages(
                GuestPhysAddr::from(ipa),
                PhysAddr::from(0 as usize),
                size,
                flags as usize,
                true,
            )?;

        Ok(pa)
    }
}
