use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::realm::mm::IPATranslation;
use monitor::realm::vcpu::VCPU;
use monitor::realm::vm::VM;
use monitor::smc;
use monitor::error::{Error, ErrorKind};

use crate::config::PAGE_SIZE;
use crate::exception::trap::syndrome::{Fault, Syndrome};
use crate::realm;
use crate::realm::context::Context;
use crate::realm::mm::page_table::pte;
use crate::realm::mm::stage2_translation::Stage2Translation;
use crate::realm::mm::translation_granule_4k::RawPTE;
use crate::helper;
use crate::helper::bits_in_reg;
use crate::helper::ESR_EL2;
use crate::helper::VTTBR_EL2;
use crate::config;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::mutex::Mutex;
use spinning_top::Spinlock;

type MutexVM = Arc<Mutex<VM<Context>>>;
type VMMap = BTreeMap<usize, MutexVM>;
static VMS: Spinlock<(usize, VMMap)> = Spinlock::new((0, BTreeMap::new()));

pub fn realm_enter() -> [usize; 4] {
    unsafe {
        if let Some(vcpu) = realm::vcpu::current() {
            if vcpu.is_vm_dead() {
                vcpu.from_current();
            } else {
                vcpu.vm.upgrade().map(|vm| {
                    vm.lock().page_table.lock().clean();
                });
                return helper::rmm_exit([0; 4]);
            }
        }
        [0, 0, 0, 0]
    }
}

pub fn realm_exit() {
    unsafe {
        if let Some(vcpu) = realm::vcpu::current() {
            vcpu.from_current();
        }
    }
}

#[derive(Debug)]
pub struct VMManager;

impl VMManager {
    pub fn new() -> &'static VMManager{
        &VMManager {}
    }
}

impl monitor::realm::vm::VMControl for VMManager {
    fn create(&self) -> Result<usize, &str> {
        let mut vms = VMS.lock();
        let id = vms.0;
        let s2_table = Arc::new(Mutex::new(
            Box::new(Stage2Translation::new()) as Box<dyn IPATranslation>
        ));
        let vttbr = bits_in_reg(VTTBR_EL2::VMID, id as u64)
            | bits_in_reg(VTTBR_EL2::BADDR, s2_table.lock().get_base_address() as u64);

        let vm = VM::new(id, s2_table);

        vm.lock()
            .vcpus
            .iter()
            .for_each(|vcpu: &Arc<Mutex<VCPU<Context>>>| {
                vcpu.lock().context.sys_regs.vttbr = vttbr
            });

        vms.0 += 1;
        vms.1.insert(id, vm.clone());
        Ok(id)
    }

    fn create_vcpu(&self, id: usize) -> Result<usize, Error> {
        VMS.lock().1.get(&id).map(|vm| Arc::clone(vm))
            .ok_or(Error::new(ErrorKind::NotConnected))?
            .lock()
            .create_vcpu()
    }

    fn remove(&self, id: usize) -> Result<(), &str> {
        VMS.lock()
            .1
            .remove(&id)
            .ok_or(Error::new(ErrorKind::NotConnected))?;
        Ok(())
    }

    fn run(&self, id: usize, vcpu: usize, incr_pc: usize) -> Result<[usize; 4], &str> {
        let get = |id: usize| VMS.lock().1.get(&id).map(|vm| Arc::clone(vm));
        if incr_pc == 1 {
            get(id).ok_or("Not exist VM")?
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
            get(id).ok_or("Not exist VM")?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or("Not exist VCPU")?
                .lock()
                .context
                .elr
        );

        get(id).ok_or("Not exist VM")?
            .lock()
            .switch_to(vcpu)?;

        trace!("Switched to VCPU {} on VM {}", vcpu, id);
        let ret = realm_enter();

        realm_exit();
        Ok(ret)
    }

    fn map(&self, id: usize, guest: usize, phys: usize, size: usize, prot: usize) -> Result<(), &str> {
        let mut flags = 0;
        let mut realm_pas = true;
        //FIXME: temporary
        unsafe {
            if let Some(vcpu) = realm::vcpu::current() {
                let esr = vcpu.context.sys_regs.esr_el2 as u32;
                info!("elr_el2 at {:#X}", vcpu.context.elr);
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

        VMS.lock().1.get(&id).map(|vm| Arc::clone(vm))
            .ok_or("Not exist VM")?
            .lock()
            .page_table
            .lock()
            .set_pages(
                GuestPhysAddr::from(guest),
                PhysAddr::from(phys),
                size,
                flags as usize,
            );

        let cmd = usize::from(config::Code::MarkRealm);
        let mut arg = [phys, 0, 0, 0];
        let mut remain = size;
        while remain > 0 {
            if (flags & helper::bits_in_reg(RawPTE::NS, 0b1)) == 0 {
                let ret = smc::call(cmd, arg)[0];
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
        VMS.lock().1.get(&id).map(|vm| Arc::clone(vm))
            .ok_or("Not exist VM")?
            .lock()
            .page_table
            .lock()
            .unset_pages(GuestPhysAddr::from(guest), size);

        //TODO change GPT to nonsecure
        //TODO zeroize memory
        Ok(())
    }

    fn set_reg(&self, id: usize, vcpu: usize, register: usize, value: usize) -> Result<(), &str> {
        let get = |id: usize| VMS.lock().1.get(&id).map(|vm| Arc::clone(vm));
        match register {
            0..=30 => {
                get(id)
                    .ok_or("Not exist VM")?
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
                get(id)
                    .ok_or("Not exist VM")?
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
                get(id)
                    .ok_or("Not exist VM")?
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
        let get = |id: usize| VMS.lock().1.get(&id).map(|vm| Arc::clone(vm));
        match register {
            0..=30 => {
                let value = get(id)
                    .ok_or("Not exist VM")?
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
                let value = get(id)
                    .ok_or("Not exist VM")?
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