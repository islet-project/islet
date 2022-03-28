use core::fmt::Debug;

pub trait IPATranslation: Debug + Send + Sync {
    fn get_vttbr(&self, vmid: usize) -> u64;
}
