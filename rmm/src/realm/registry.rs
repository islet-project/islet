use crate::realm::Realm;

use crate::realm::context::Context;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use spin::mutex::Mutex;
use spinning_top::Spinlock;

type RealmMutex = Arc<Mutex<Realm<Context>>>;
type RealmMap = BTreeMap<usize, RealmMutex>;
pub static RMS: Spinlock<(usize, RealmMap)> = Spinlock::new((0, BTreeMap::new()));

pub fn get_realm(id: usize) -> Option<RealmMutex> {
    RMS.lock().1.get(&id).map(Arc::clone)
}
