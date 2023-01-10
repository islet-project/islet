use monitor::error::{Error, ErrorKind};
use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::realm::mm::IPATranslation;
use monitor::realm::vcpu::VCPU;
use monitor::realm::vm::VM;

use crate::config::PAGE_SIZE;
use crate::exception::trap::syndrome::{Fault, Syndrome};
use crate::helper;
use crate::helper::ESR_EL2;
use crate::realm;
use crate::realm::context::Context;
use crate::realm::mm::page_table::pte;
use crate::realm::mm::stage2_translation::Stage2Translation;
use crate::realm::mm::translation_granule_4k::RawPTE;

use crate::helper::bits_in_reg;
use crate::helper::VTTBR_EL2;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::mutex::Mutex;
use spinning_top::Spinlock;

use crate::smc;

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

#[derive(Copy, Clone, Debug)]
pub struct VMManager {
    pub id: usize,
}

pub trait VMControl {
    fn new(&mut self) -> MutexVM;
    fn get(&self) -> Option<MutexVM>;
    fn remove(&self) -> Result<(), Error>;
    fn run(&self, vcpu: usize, incr_pc: usize) -> Result<[usize; 4], &str>;
}

impl VMControl for VMManager {
    fn new(&mut self) -> MutexVM {
        let mut vms = VMS.lock();

        //TODO limit id to fit in VMID (16 bits)
        self.id = vms.0;

        let s2_table = Arc::new(Mutex::new(
            Box::new(Stage2Translation::new()) as Box<dyn IPATranslation>
        ));

        let vttbr = bits_in_reg(VTTBR_EL2::VMID, self.id as u64)
            | bits_in_reg(VTTBR_EL2::BADDR, s2_table.lock().get_base_address() as u64);
        let vm = VM::new(self.id, s2_table);

        vm.lock()
            .vcpus
            .iter()
            .for_each(|vcpu: &Arc<Mutex<VCPU<Context>>>| {
                vcpu.lock().context.sys_regs.vttbr = vttbr
            });

        vms.0 += 1;
        vms.1.insert(self.id, vm.clone());

        vm
    }

    fn get(&self) -> Option<MutexVM> {
        VMS.lock().1.get(&self.id).map(|vm| Arc::clone(vm))
    }

    fn remove(&self) -> Result<(), Error> {
        VMS.lock()
            .1
            .remove(&self.id)
            .ok_or(Error::new(ErrorKind::NotConnected))?;
        Ok(())
    }

    fn run(&self, vcpu: usize, incr_pc: usize) -> Result<[usize; 4], &str> {
        if incr_pc == 1 {
            self.get()
                .ok_or("Not exist VM")?
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
            self.get()
                .ok_or("Not exist VM")?
                .lock()
                .vcpus
                .get(vcpu)
                .ok_or("Not exist VCPU")?
                .lock()
                .context
                .elr
        );

        self.get().ok_or("Not exist VM")?.lock().switch_to(vcpu)?;

        trace!("Switched to VCPU {} on VM {}", vcpu, self.id);
        let ret = realm_enter();

        realm_exit();
        Ok(ret)
    }
}

pub trait VMMemory {
    fn map(&self, guest: usize, phys: usize, size: usize, prot: usize) -> Result<(), &str>;
    fn unmap(&self, guest: usize, size: usize) -> Result<(), &str>;
}

impl VMMemory for VMManager {
    fn map(&self, guest: usize, phys: usize, size: usize, prot: usize) -> Result<(), &str> {
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

        self.get()
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

        let cmd = usize::from(smc::Code::MarkRealm);
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

    fn unmap(&self, guest: usize, size: usize) -> Result<(), &str> {
        self.get()
            .ok_or("Not exist VM")?
            .lock()
            .page_table
            .lock()
            .unset_pages(GuestPhysAddr::from(guest), size);

        //TODO change GPT to nonsecure
        //TODO zeroize memory
        Ok(())
    }
}

pub trait VMRegister {
    fn set_reg(&self, vcpu: usize, register: usize, value: usize) -> Result<(), &str>;
    fn get_reg(&self, vcpu: usize, register: usize) -> Result<(), &str>;
}

impl VMRegister for VMManager {
    fn set_reg(&self, vcpu: usize, register: usize, value: usize) -> Result<(), &str> {
        match register {
            0..=30 => {
                self.get()
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
                self.get()
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
                self.get()
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

    fn get_reg(&self, vcpu: usize, register: usize) -> Result<(), &str> {
        match register {
            0..=30 => {
                self.get()
                    .ok_or("Not exist VM")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .gp_regs[register];
                Ok(())
            }
            31 => {
                self.get()
                    .ok_or("Not exist VM")?
                    .lock()
                    .vcpus
                    .get(vcpu)
                    .ok_or("Not exist VCPU")?
                    .lock()
                    .context
                    .elr;
                Ok(())
            }
            _ => Err("Failed"),
        }?;
        Ok(())
    }
}
