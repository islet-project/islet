use monitor::error::{Error, ErrorKind};
use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::realm::mm::IPATranslation;
use monitor::realm::vcpu::VCPU;
use monitor::realm::Realm;

use crate::config::PAGE_SIZE;
use crate::exception::trap::syndrome::{Fault, Syndrome};
use crate::helper;
use crate::helper::bits_in_reg;
use crate::helper::ESR_EL2;
use crate::helper::VTTBR_EL2;
use crate::realm;
use crate::realm::context::Context;
use crate::realm::mm::page_table::pte;
use crate::realm::mm::stage2_translation::Stage2Translation;
use crate::realm::mm::translation_granule_4k::RawPTE;
use crate::smc::SMC;
use monitor::smc::{self, Caller};

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
        let mut realm_pas = true;
        //FIXME: temporary
        unsafe {
            if let Some(vcpu) = realm::vcpu::current() {
                let esr = vcpu.context.sys_regs.esr_el2 as u32;
                // share all data pages except those had s2 permission fault with s1ptw set
                match Syndrome::from(esr) {
                    Syndrome::DataAbort(fault) => {
                        realm_pas = false;
                        if esr & ESR_EL2::S1PTW as u32 != 0 {
                            match fault {
                                Fault::Permission { level } => {
                                    realm_pas = true;
                                    debug!("Data permission fault at {}", level);
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        if realm_pas == false {
            flags |= helper::bits_in_reg(RawPTE::NS, 0b1);
        }

        // TODO:  define bit mask
        flags |= helper::bits_in_reg(RawPTE::S2AP, pte::permission::RW);
        if prot & 0b100 == 0b100 {
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

        let smc = SMC::new();
        let cmd = smc.convert(smc::Code::MarkRealm);
        let mut arg = [phys, 0, 0, 0];
        let mut remain = size;
        while remain > 0 {
            if (flags & helper::bits_in_reg(RawPTE::NS, 0b1)) == 0 {
                let ret = smc.call(cmd, arg)[0];
                if ret != 0 {
                    //Just show a warn message not return fail
                    warn!("failed to set GPT {:X}", arg[0]);
                }
            }
            arg[0] += PAGE_SIZE;
            remain -= PAGE_SIZE;
        }
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
}
