#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TrapFrame {
    pub _res: u64,
    pub elr: u64,
    pub spsr: u64,
    pub regs: [u64; 31],
}
