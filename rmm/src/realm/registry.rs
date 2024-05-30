use crate::rmi::error::Error;
use alloc::collections::BTreeSet;
use spinning_top::Spinlock;

pub static VMID_SET: Spinlock<BTreeSet<usize>> = Spinlock::new(BTreeSet::new());

pub fn remove(id: usize) -> Result<(), Error> {
    VMID_SET
        .lock()
        .remove(&id)
        .then_some(())
        .ok_or(Error::RmiErrorInput)
}
