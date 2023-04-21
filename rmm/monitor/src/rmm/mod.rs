pub mod address;

pub type PageMap = &'static dyn RmmPage;

pub trait RmmPage {
    fn map(&self, phys: [usize; 4]) -> Result<(), &str>;
    fn unmap(&self, phys: [usize; 4]) -> Result<(), &str>;
}
