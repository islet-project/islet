pub mod lower;
pub mod trap;

core::arch::global_asm!(include_str!("vectors.s"));
extern "C" {
    pub static mut vectors: u64;
}
