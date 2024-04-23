use alloc::collections::BTreeSet;
use spinning_top::Spinlock;

pub static VMID_SET: Spinlock<BTreeSet<usize>> = Spinlock::new(BTreeSet::new());
