use monitor::error::{Error, ErrorKind};
use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::realm::mm::IPATranslation;
use monitor::realm::vcpu::VCPU;
use monitor::realm::Realm;
use monitor::rmi::rec::run::Run;
use monitor::rmi::MapProt;

use crate::gic::{GIC_FEATURES, ICH_HCR_EL2_EOI_COUNT_MASK, ICH_HCR_EL2_NS_MASK};
use crate::helper;
use crate::helper::bits_in_reg;
use crate::helper::VTTBR_EL2;
use crate::realm;
use crate::realm::context::Context;
use crate::realm::mm::page_table::pte;
use crate::realm::mm::stage2_translation::Stage2Translation;
use crate::realm::mm::translation_granule_4k::RawPTE;

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
    fn create_realm(&self) -> Result<usize, &str> {
        let mut rms = RMS.lock();
        let id = rms.0;
        let s2_table = Arc::new(Mutex::new(
            Box::new(Stage2Translation::new()) as Box<dyn IPATranslation>
        ));
        let realm = Realm::new(id, s2_table);

        rms.0 += 1;
        rms.1.insert(id, realm.clone());
        Ok(id)
    }

    fn create_vcpu(&self, id: usize) -> Result<usize, Error> {
        let realm = get_realm(id).ok_or(Error::new(ErrorKind::NotConnected))?;

        let page_table = realm.lock().page_table.lock().get_base_address();
        let vttbr = bits_in_reg(VTTBR_EL2::VMID, id as u64)
            | bits_in_reg(VTTBR_EL2::BADDR, page_table as u64);

        let _vcpu = VCPU::new(realm.clone());
        _vcpu.lock().context.sys_regs.vttbr = vttbr;

        realm.lock().vcpus.push(_vcpu);
        let vcpuid = realm.lock().vcpus.len() - 1;
        Ok(vcpuid)
    }

    fn remove(&self, id: usize) -> Result<(), &str> {
        RMS.lock()
            .1
            .remove(&id)
            .ok_or(Error::new(ErrorKind::NotConnected))?;
        Ok(())
    }

    fn run(&self, id: usize, vcpu: usize, incr_pc: usize) -> Result<[usize; 4], &str> {
        if incr_pc == 1 {
            get_realm(id)
                .ok_or("Not exist Realm")?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or("Not exist VCPU")?
                .lock()
                .context
                .elr += 4;
        }
        debug!(
            "resuming: {:#x}",
            get_realm(id)
                .ok_or("Not exist Realm")?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or("Not exist VCPU")?
                .lock()
                .context
                .elr
        );

        get_realm(id)
            .ok_or("Not exist Realm")?
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
    ) -> Result<(), &str> {
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
            .ok_or("Not exist Realm")?
            .lock()
            .page_table
            .lock()
            .set_pages(
                GuestPhysAddr::from(guest),
                PhysAddr::from(phys),
                size,
                flags as usize,
            );

        Ok(())
    }

    fn unmap(&self, id: usize, guest: usize, size: usize) -> Result<(), &str> {
        get_realm(id)
            .ok_or("Not exist Realm")?
            .lock()
            .page_table
            .lock()
            .unset_pages(GuestPhysAddr::from(guest), size);

        //TODO change GPT to nonsecure
        //TODO zeroize memory
        Ok(())
    }

    fn set_reg(&self, id: usize, vcpu: usize, register: usize, value: usize) -> Result<(), &str> {
        match register {
            0..=30 => {
                get_realm(id)
                    .ok_or("Not exist Realm")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .gp_regs[register] = value as u64;
                Ok(())
            }
            31 => {
                get_realm(id)
                    .ok_or("Not exist Realm")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .elr = value as u64;
                Ok(())
            }
            32 => {
                get_realm(id)
                    .ok_or("Not exist Realm")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .spsr = value as u64;
                Ok(())
            }
            _ => Err("Failed"),
        }?;
        Ok(())
    }

    fn get_reg(&self, id: usize, vcpu: usize, register: usize) -> Result<usize, &str> {
        match register {
            0..=30 => {
                let value = get_realm(id)
                    .ok_or("Not exist Realm")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .gp_regs[register];
                Ok(value as usize)
            }
            31 => {
                let value = get_realm(id)
                    .ok_or("Not exist Realm")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .elr;
                Ok(value as usize)
            }
            _ => Err("Failed"),
        }
    }

    fn receive_gic_state_from_host(&self, id: usize, vcpu: usize, run: &Run) -> Result<(), &str> {
        let realm = get_realm(id).ok_or("Not exist Realm")?;
        let locked_realm = realm.lock();
        let vcpu = locked_realm.vcpus.get(vcpu).ok_or("Not exist VCPU")?;
        let gic_state = &mut vcpu.lock().context.gic_state;
        let nr_lrs = GIC_FEATURES.nr_lrs;

        gic_state.ich_lr_el2[..nr_lrs].copy_from_slice(&unsafe { run.entry_gic_lrs() }[..nr_lrs]);
        gic_state.ich_hcr_el2 &= !ICH_HCR_EL2_NS_MASK;
        gic_state.ich_hcr_el2 |= (unsafe { run.entry_gic_hcr() } | ICH_HCR_EL2_NS_MASK);
        Ok(())
    }

    fn send_gic_state_to_host(&self, id: usize, vcpu: usize, run: &mut Run) -> Result<(), &str> {
        let realm = get_realm(id).ok_or("Not exist Realm")?;
        let mut locked_realm = realm.lock();
        let vcpu = locked_realm.vcpus.get_mut(vcpu).ok_or("Not exist VCPU")?;
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
}
