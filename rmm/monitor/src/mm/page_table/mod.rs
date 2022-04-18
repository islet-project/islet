use super::address::PhysAddr;

pub trait Level {
    const THIS_LEVEL: usize;
}

pub trait HasSubtable: Level {
    type NextLevel;
}

pub trait Entry {
    fn new() -> Self;
    fn is_valid(&self) -> bool;
    fn clear(&mut self);

    fn address(&self, level: usize) -> Option<PhysAddr>;
    fn set(&mut self, addr: PhysAddr, flags: u64);
}
