#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    pub elr: u64,
    pub spsr: u64,
    pub sp0: u64,
    pub tpidr0: u64,
    pub sp1: u64,
    pub tpidr1: u64,
    pub regs: [u64; 31],
}
