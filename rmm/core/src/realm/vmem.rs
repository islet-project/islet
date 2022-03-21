use core::fmt::Debug;

pub trait IPATranslation: Debug + Send + Sync {
    fn set_mmu(&mut self);
}
