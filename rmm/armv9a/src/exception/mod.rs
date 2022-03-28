pub mod trap;

global_asm!(include_str!("vectors.s"));
extern "C" {
    pub static mut vectors: u64;
}
