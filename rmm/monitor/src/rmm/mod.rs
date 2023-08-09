pub mod address;

#[macro_use]
pub mod granule;

pub type PageMap = &'static dyn RmmPage;

pub trait RmmPage {
    fn map(&self, phys: usize, secure: bool) -> bool;
    fn unmap(&self, phys: usize) -> bool;
}
