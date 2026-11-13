use crate::config::PAGE_SIZE;
use aarch64_cpu::registers::{Readable, Writeable};
use armv9a::regs::{CPTR_EL2, SMCR_EL2, SVCR, ZCR_EL1, ZCR_EL2};
use armv9a::InMemoryRegister;
use core::alloc::Layout;
use core::arch::asm;
use core::array::from_fn;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use super::Rec;
use crate::asm::isb;
use crate::config::NUM_OF_CPU;
use crate::const_assert_eq;
use crate::cpu::get_cpu_id;
use crate::rmi::error::Error;
use crate::simd::{sme_en, sve_en as host_sve_en, SimdConfig, SVE_VQ_ARCH_MAX};

use vmsa::address::VirtAddr;

/// Alignment (in bytes) required for the SIMD context allocation (`simd_va`)
/// and the FPU/SVE register area (`FpuRegs` or SVE `Z`/`P`/`FFR` register
/// banks) that follows `SimdSysRegs`. Both `ldp`/`stp qN` and SVE `MUL VL`
/// addressing require 16-byte alignment. Note that this 16 is a byte-alignment
/// requirement; it is unrelated to `NUM_PREDICATE_REGS` or `BYTES_PER_QUAD`,
/// which merely happen to equal 16.
const FPU_SVE_REGS_ALIGN: usize = 16;

/// SIMD Context Layout within the Contiguous AUX Granule Mapping (`simd_va`):
///
/// The host obtains the number of required AUX granules via `RMI_REC_AUX_COUNT`
/// and passes them at `RMI_REC_CREATE`. `Rec::setup_aux_memory()`
/// maps all `params.aux[0..num_aux]` granules ONCE into a single contiguous
/// virtual address range (`aux_va_start`, `aux_va_size`) and initializes the
/// REC's [`AuxBumpAllocator`]. During `Rec::init()`, `init_simd()` allocates
/// the SIMD context region (`simd_va`) from that bump allocator via
/// `Rec::alloc_aux()` with 16-byte alignment (`FPU_SVE_REGS_ALIGN`).
///
/// That granule count is computed by `aux_granules_needed()`: it is
/// `EXTRA_AUX_GRANULES_COUNT` plus the number of granules needed to hold the
/// layout described below.
///
/// The `SimdSysRegs` control/status header is placed at offset `0` of
/// `simd_va`, and the FPU/SVE register area (`FpuRegs` when SVE is disabled,
/// or the SVE `Z`/`P`/`FFR` register banks when SVE is enabled) is placed
/// immediately after `SimdSysRegs` at `SimdContext::FPU_SVE_REGS_OFFSET`:
///
/// ```text
///   simd_va + 0x00 +-----------------------------------------------------------+
///                  | SimdSysRegs (control & status register header, 48 bytes)  |
///                  |   - svcr     : u64   (SME streaming vector control)       |
///                  |   - zcr_el2  : u64   (EL2 SVE vector length cap)          |
///                  |   - zcr_el12 : u64   (Realm EL1 SVE vector length)        |
///                  |   - fpsr     : u64   (Floating-point status register)     |
///                  |   - fpcr     : u64   (Floating-point control register)    |
///                  |   - (8 bytes padding to 16-byte alignment)                |
///   simd_va + 0x30 +-----------------------------------------------------------+
///                  | FPU/SVE Register Area (at FPU_SVE_REGS_OFFSET = 48)       |
///                  |                                                           |
///                  | * When SVE is disabled (`cfg.sve_en == false`):           |
///                  |   - FpuRegs (`q[0..32]`): 32 * 16 = 512 bytes             |
///                  |                                                           |
///                  | * When SVE is enabled (`cfg.sve_en == true`):             |
///                  |   - Z0..Z31 : 32 * VL bytes       (offset + 0)            |
///                  |   - P0..P15 : 16 * (VL / 8) bytes (offset + 32 * VL)      |
///                  |   - FFR     : 1 * (VL / 8) bytes  (offset + 34 * VL)      |
///                  |   (where VL = (sve_vq + 1) * 16 bytes; Q0..Q31 alias the  |
///                  |    lower 128 bits of Z0..Z31 so no separate FpuRegs is    |
///                  |    needed, and the buffer may span multiple 4 KB AUX      |
///                  |    granules across the contiguous VA mapping)             |
///                  +-----------------------------------------------------------+
/// ```
#[derive(Default, Debug)]
pub struct SimdContext {
    pub is_used: bool,
    pub is_saved: bool,

    pub cfg: SimdConfig,
    /// Virtual address of the SIMD context region allocated from the REC's
    /// [`AuxBumpAllocator`] inside the contiguous multi-granule AUX mapping.
    pub simd_va: Option<VirtAddr>,
}

impl SimdContext {
    /// Byte offset of the FPU/SVE register area (`FpuRegs` when SVE is
    /// disabled, or the SVE `Z`, `P`, and `FFR` banks when SVE is enabled)
    /// within the SIMD context region (`simd_va`).
    /// It starts right after `SimdSysRegs`, aligned to `FPU_SVE_REGS_ALIGN`.
    const FPU_SVE_REGS_OFFSET: usize =
        (core::mem::size_of::<SimdSysRegs>() + FPU_SVE_REGS_ALIGN - 1) & !(FPU_SVE_REGS_ALIGN - 1);

    /// Base virtual address of the allocated SIMD context region.
    #[inline(always)]
    fn base_va(&self) -> usize {
        self.simd_va.expect("SIMD context is not mapped").as_usize()
    }

    /// Reference to the `SimdSysRegs` control/status header at offset `0` of
    /// the allocated SIMD context region (`simd_va`).
    ///
    /// # Safety
    /// The caller must ensure `self.simd_va` points to the base of a valid AUX
    /// allocation large enough to hold `SimdSysRegs`.
    #[inline(always)]
    unsafe fn sys_regs(&self) -> &SimdSysRegs {
        unsafe { &*(self.base_va() as *const SimdSysRegs) }
    }

    /// Mutable reference to the `SimdSysRegs` control/status header at offset
    /// `0` of the allocated SIMD context region (`simd_va`).
    ///
    /// # Safety
    /// The caller must ensure `self.simd_va` points to the base of a valid AUX
    /// allocation large enough to hold `SimdSysRegs`.
    #[inline(always)]
    unsafe fn sys_regs_mut(&mut self) -> &mut SimdSysRegs {
        unsafe { &mut *(self.base_va() as *mut SimdSysRegs) }
    }

    /// Virtual address of the FPU/SVE register area (`FpuRegs` or SVE
    /// register buffer) that immediately follows the `SimdSysRegs` header
    /// in the allocated SIMD context region.
    #[inline(always)]
    fn fpu_sve_regs_va(&self) -> VirtAddr {
        VirtAddr::from(self.base_va() + Self::FPU_SVE_REGS_OFFSET)
    }

    /// Reference to the FPU register bank (`Q0..Q31`) stored immediately
    /// after the `SimdSysRegs` header when SVE is disabled.
    ///
    /// # Safety
    /// The caller must ensure `self.simd_va` points to the base of a valid AUX
    /// allocation large enough to hold `SimdSysRegs` + `FpuRegs`.
    #[inline(always)]
    unsafe fn fpu_regs(&self) -> &FpuRegs {
        unsafe { &*(self.fpu_sve_regs_va().as_usize() as *const FpuRegs) }
    }

    /// Mutable reference to the FPU register bank (`Q0..Q31`) stored
    /// immediately after the `SimdSysRegs` header when SVE is disabled.
    ///
    /// # Safety
    /// The caller must ensure `self.simd_va` points to the base of a valid AUX
    /// allocation large enough to hold `SimdSysRegs` + `FpuRegs`.
    #[inline(always)]
    unsafe fn fpu_regs_mut(&mut self) -> &mut FpuRegs {
        unsafe { &mut *(self.fpu_sve_regs_va().as_usize() as *mut FpuRegs) }
    }
}

/// Per-context SIMD/SVE control and status system registers.
///
/// For a REC, this header lives at offset `0` of the allocated SIMD context
/// region (`simd_va`), and the FPU/SVE registers (`FpuRegs` when SVE is
/// disabled, or the SVE `Z`, `P`, and `FFR` banks when SVE is enabled) are
/// stored immediately after this struct at `SimdContext::FPU_SVE_REGS_OFFSET`.
#[repr(C, align(16))]
#[derive(Default, Debug)]
struct SimdSysRegs {
    // SME specific Streaming vector control register
    svcr: u64,
    // SVE vector length control registers
    zcr_el2: u64,
    zcr_el12: u64,
    // Floating-point status and control system registers
    fpsr: u64,
    fpcr: u64,
}
const_assert_eq!(core::mem::align_of::<SimdSysRegs>(), FPU_SVE_REGS_ALIGN);
const_assert_eq!(SimdContext::FPU_SVE_REGS_OFFSET % FPU_SVE_REGS_ALIGN, 0);

impl SimdSysRegs {
    fn new() -> Self {
        Default::default()
    }
}

// FPU registers (Q0..Q31).
// Stored right after `SimdSysRegs` (at `SimdContext::FPU_SVE_REGS_OFFSET`)
// when SVE is disabled for the REC, or in `NsSimdContext::fpu` for the NS
// world on a non-SVE PE.
const NUM_FPU_REGS: usize = 32;
#[repr(C, align(16))]
#[derive(Default, Debug)]
struct FpuRegs {
    q: [u128; NUM_FPU_REGS],
}
const_assert_eq!(core::mem::align_of::<FpuRegs>(), FPU_SVE_REGS_ALIGN);

// SVE registers
const NUM_VECTOR_REGS: usize = 32;
const NUM_PREDICATE_REGS: usize = 16;
// Architectural maximum number of 128-bit quadwords in a Z register:
// SVE supports VL from 128 bits up to 2048 bits.
const ARCH_MAX_QUADS: usize = (SVE_VQ_ARCH_MAX + 1) as usize; // 16
/// Bytes in one 128-bit quadword: the quantum of the SVE vector length
/// (VL in bytes = (VQ + 1) * BYTES_PER_QUAD).
const BYTES_PER_QUAD: usize = 16;
/// Number of bits in a byte.
const BITS_PER_BYTE: usize = 8;
/// Predicate bytes per quadword: one predicate bit per byte element.
const BYTES_PER_PREDICATE_QUAD: usize = BYTES_PER_QUAD / BITS_PER_BYTE; // 2
/// Predicate quadwords that fit in one u128 storage word.
const PREDICATE_QUADS_PER_WORD: usize = BYTES_PER_QUAD / BYTES_PER_PREDICATE_QUAD; // 8

/// Non-secure world SVE register banks, sized for the architectural maximum
/// vector length (VL = 2048 bits). The register data is accessed with a
/// MUL VL stride for the currently active VL, so the storage must hold:
/// - 32 * VL bytes for the Z registers,
/// - 16 * VL/8 bytes for the P registers,
/// - VL/8 bytes for the FFR register.
#[derive(Debug)]
struct NsSveRegs {
    // NUM_VECTOR_REGS Z registers, each up to ARCH_MAX_QUADS quads
    // (one quad per u128)
    z: [u128; NUM_VECTOR_REGS * ARCH_MAX_QUADS],
    // NUM_PREDICATE_REGS P registers, each up to
    // (ARCH_MAX_QUADS * BYTES_PER_PREDICATE_QUAD) bytes
    p: [u128; NUM_PREDICATE_REGS * (ARCH_MAX_QUADS / PREDICATE_QUADS_PER_WORD)],
    // 1 FFR register, up to (ARCH_MAX_QUADS * BYTES_PER_PREDICATE_QUAD) bytes
    ffr: [u128; ARCH_MAX_QUADS / PREDICATE_QUADS_PER_WORD],
}

// [u128; N] with N > 32 does not implement Default, so implement it manually.
impl Default for NsSveRegs {
    fn default() -> Self {
        Self {
            z: from_fn(|_| 0),
            p: from_fn(|_| 0),
            ffr: from_fn(|_| 0),
        }
    }
}

/// Per-CPU Non-Secure world SIMD/SVE/FPU saved context.
#[derive(Default, Debug)]
struct NsSimdContext {
    sys_regs: SimdSysRegs,
    fpu: FpuRegs,
    sve: NsSveRegs,
}

lazy_static! {
    // Non-secure world SIMD/SVE/FPU context, per CPU
    static ref NS_SIMD_CTX: [Mutex<NsSimdContext>; NUM_OF_CPU] =
        from_fn(|_| Mutex::new(NsSimdContext::default()));
}

/// Calculate the number of pages needed for a given size in bytes
fn pages_for_size(size: usize) -> usize {
    size.div_ceil(PAGE_SIZE)
}

/// Calculate the SVE register data size in bytes for a given VQ
/// (Vector Quadwords). VQ is the zero-based maximum number of 128-bit
/// quadwords in a Z register, i.e. VL = (VQ + 1) * 128 bits.
/// The SVE data consists of:
/// - 32 Z registers, each of size (VQ + 1) * 16 bytes
/// - 16 P registers, each of size (VQ + 1) * 2 bytes
/// - 1 FFR register, of size (VQ+1) * 2 bytes
fn sve_data_size(vq: u64) -> usize {
    // VL in bytes: (VQ + 1) quadwords of BYTES_PER_QUAD bytes each
    let vl_bytes = ((vq + 1) as usize) * BYTES_PER_QUAD;
    // P registers and the FFR are one bit per byte element: VL / 8 bytes
    let p_reg_size = vl_bytes / BITS_PER_BYTE;

    NUM_VECTOR_REGS * vl_bytes + NUM_PREDICATE_REGS * p_reg_size + p_reg_size
}

/// Byte offset of the P register bank inside the per-REC SVE buffer for a
/// vector length of `vl_bytes` (`MUL VL` layout: the P bank immediately
/// follows the Z bank). Shared by `save_rec_sve()` and `restore_rec_sve()`
/// so both agree on the buffer layout.
const fn rec_sve_p_offset(vl_bytes: usize) -> usize {
    NUM_VECTOR_REGS * vl_bytes
}

/// Byte offset of the FFR inside the per-REC SVE buffer (immediately after
/// the P register bank) for a vector length of `vl_bytes`.
const fn rec_sve_ffr_offset(vl_bytes: usize) -> usize {
    rec_sve_p_offset(vl_bytes) + NUM_PREDICATE_REGS * (vl_bytes / BITS_PER_BYTE)
}

/// Calculate the total number of 4KB AUX granules needed for a REC when SVE
/// is enabled for a given VQ (covering both the `SimdSysRegs` header at
/// offset 0 and the SVE register data at `SimdContext::FPU_SVE_REGS_OFFSET`).
fn sve_pages_needed(vq: u64) -> usize {
    pages_for_size(SimdContext::FPU_SVE_REGS_OFFSET + sve_data_size(vq))
}

/// Total size in bytes of the storage used for the FP/SIMD (non-SVE) context:
/// the `SimdSysRegs` header followed by the `FpuRegs` (`Q0..Q31`) bank at
/// `SimdContext::FPU_SVE_REGS_OFFSET`.
fn fpu_data_size() -> usize {
    SimdContext::FPU_SVE_REGS_OFFSET + core::mem::size_of::<FpuRegs>()
}

/// Calculate the total number of 4KB AUX granules needed for a REC when SVE
/// is disabled (covering both the `SimdSysRegs` header at offset 0 and the
/// `FpuRegs` register data at `SimdContext::FPU_SVE_REGS_OFFSET`).
fn fpu_pages_needed() -> usize {
    pages_for_size(fpu_data_size())
}

/// AUX granules that are always reserved for a REC on top of the granules
/// that hold the SIMD context itself (see [`aux_granules_needed`]).
/// They are needed to pass ACS tests.
const EXTRA_AUX_GRANULES_COUNT: usize = 1;

/// Calculate the total number of AUX granules a REC needs for its SIMD
/// context, i.e. the value reported by `RMI_REC_AUX_COUNT` and validated
/// against `params.num_aux` at `RMI_REC_CREATE`.
///
/// It is [`EXTRA_AUX_GRANULES_COUNT`] plus the number of granules needed to
/// store the SIMD context in the contiguous AUX mapping:
/// - SVE enabled: the `SimdSysRegs` header plus the SVE `Z`/`P`/`FFR` banks
///   for the configured VQ (see [`sve_pages_needed`]).
/// - SVE disabled: the `SimdSysRegs` header plus the `FpuRegs` (`Q0..Q31`)
///   bank (see [`fpu_pages_needed`]).
pub fn aux_granules_needed(simd_cfg: &SimdConfig) -> usize {
    let simd_pages = if simd_cfg.sve_en {
        sve_pages_needed(simd_cfg.sve_vq)
    } else {
        fpu_pages_needed()
    };

    EXTRA_AUX_GRANULES_COUNT + simd_pages
}

/// Total size in bytes of the SIMD context region (`SimdSysRegs` header at
/// offset 0 plus the FPU or SVE register area at `SimdContext::FPU_SVE_REGS_OFFSET`).
fn simd_context_size(simd_cfg: &SimdConfig) -> usize {
    if simd_cfg.sve_en {
        SimdContext::FPU_SVE_REGS_OFFSET + sve_data_size(simd_cfg.sve_vq)
    } else {
        fpu_data_size()
    }
}

/// Memory layout (size and 16-byte alignment) required for the SIMD context
/// region allocated from the REC's [`AuxBumpAllocator`].
fn simd_context_layout(simd_cfg: &SimdConfig) -> Result<Layout, Error> {
    Layout::from_size_align(simd_context_size(simd_cfg), FPU_SVE_REGS_ALIGN)
        .map_err(|_| Error::RmiErrorInput)
}

// TODO: Save according to the hint in FID with SMCCCv1.3 or v1.4

// SIMD context initialization function
pub fn init_simd(rec: &mut Rec<'_>) -> Result<(), Error> {
    let simd_cfg = rec.simd_config()?;
    let sve_en = simd_cfg.sve_en;
    let sve_vq = simd_cfg.sve_vq;
    let sme_en_cfg = simd_cfg.sme_en;
    let layout = simd_context_layout(simd_cfg)?;

    let simd_ptr = rec.alloc_aux(layout).ok_or_else(|| {
        error!("Failed to allocate SIMD context from REC AUX bump allocator");
        Error::RmiErrorInput
    })?;
    let simd_va = VirtAddr::from(simd_ptr.as_ptr() as usize);

    let mut zcr_el2: u64 = 0;
    let mut svcr: u64 = 0;

    let rec_simd_ctx = &mut rec.context.simd;
    rec_simd_ctx.simd_va = Some(simd_va);
    rec_simd_ctx.is_used = false;
    rec_simd_ctx.is_saved = false;
    rec_simd_ctx.cfg.sve_en = sve_en;
    rec_simd_ctx.cfg.sve_vq = sve_vq;
    rec_simd_ctx.cfg.sme_en = sme_en_cfg;

    // Initialize SVE related fields and config registers
    if sve_en {
        zcr_el2 = ZCR_EL2::LEN.val(sve_vq).value;
    }
    if sme_en_cfg {
        svcr = 0;
    }

    // The SimdSysRegs header lives at offset 0 of `simd_va` (allocated from the
    // REC's AuxBumpAllocator with 16-byte alignment by `rec.alloc_aux()`), and
    // the FPU/SVE register area at `simd_va + SimdContext::FPU_SVE_REGS_OFFSET`
    // is already zero-initialized from granule delegation.
    let rec_simd_regs = unsafe { rec_simd_ctx.sys_regs_mut() };
    *rec_simd_regs = SimdSysRegs::new();
    rec_simd_regs.zcr_el2 = zcr_el2;
    rec_simd_regs.svcr = svcr;
    Ok(())
}

// Note: Put assembly instructions only in the functions below.
//   Do not add log messages in functions with simd features.
//   RMM doesn't maintain its own SIMD context. However, compiler
//   tries to utilize simd registers in a function with the 'neon'
//   or 'sve' feature and it may results state corruption.

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context. The q registers are
/// saved in pairs via `stp`, i.e. at fixed byte offsets of 32 bytes
/// (2 * a 16-byte register); those offsets are asm literals by design.
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "neon")]
unsafe fn save_fpu(fpu: &mut FpuRegs) {
    let addr_q: u64 = fpu.q.as_ptr() as u64;
    unsafe {
        asm!(
            "stp q0, q1, [{addr_q}]",
            "stp q2, q3, [{addr_q}, #32]",
            "stp q4, q5, [{addr_q}, #64]",
            "stp q6, q7, [{addr_q}, #96]",
            "stp q8, q9, [{addr_q}, #128]",
            "stp q10, q11, [{addr_q}, #160]",
            "stp q12, q13, [{addr_q}, #192]",
            "stp q14, q15, [{addr_q}, #224]",
            "stp q16, q17, [{addr_q}, #256]",
            "stp q18, q19, [{addr_q}, #288]",
            "stp q20, q21, [{addr_q}, #320]",
            "stp q22, q23, [{addr_q}, #352]",
            "stp q24, q25, [{addr_q}, #384]",
            "stp q26, q27, [{addr_q}, #416]",
            "stp q28, q29, [{addr_q}, #448]",
            "stp q30, q31, [{addr_q}, #480]",
            addr_q = in(reg) addr_q,
        );
    }
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "neon")]
unsafe fn save_fpu_crsr(sys_regs: &mut SimdSysRegs) {
    let fpsr: u64;
    let fpcr: u64;
    unsafe {
        asm!(
            "mrs {fpsr}, fpsr",
            "mrs {fpcr}, fpcr",
            fpsr = out(reg) fpsr,
            fpcr = out(reg) fpcr,
        );
    }
    sys_regs.fpsr = fpsr;
    sys_regs.fpcr = fpcr;
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "neon")]
unsafe fn restore_fpu(fpu: &FpuRegs) {
    let addr_q: u64 = fpu.q.as_ptr() as u64;
    unsafe {
        asm!(
            "ldp q0, q1, [{addr_q}]",
            "ldp q2, q3, [{addr_q}, #32]",
            "ldp q4, q5, [{addr_q}, #64]",
            "ldp q6, q7, [{addr_q}, #96]",
            "ldp q8, q9, [{addr_q}, #128]",
            "ldp q10, q11, [{addr_q}, #160]",
            "ldp q12, q13, [{addr_q}, #192]",
            "ldp q14, q15, [{addr_q}, #224]",
            "ldp q16, q17, [{addr_q}, #256]",
            "ldp q18, q19, [{addr_q}, #288]",
            "ldp q20, q21, [{addr_q}, #320]",
            "ldp q22, q23, [{addr_q}, #352]",
            "ldp q24, q25, [{addr_q}, #384]",
            "ldp q26, q27, [{addr_q}, #416]",
            "ldp q28, q29, [{addr_q}, #448]",
            "ldp q30, q31, [{addr_q}, #480]",
            addr_q = in(reg) addr_q,
        );
    }
}

/// # Safety
///
/// Use neon only for (re)storing Rec's simd context
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "neon")]
unsafe fn restore_fpu_crsr(sys_regs: &SimdSysRegs) {
    unsafe {
        asm!(
            "msr fpsr, {fpsr}",
            "msr fpcr, {fpcr}",
            fpsr = in(reg) sys_regs.fpsr,
            fpcr = in(reg) sys_regs.fpcr,
        );
    }
}

/// # Safety
///
/// Save the SVE register banks (`Z0..Z31`, `P0..P15`, and optionally `FFR`)
/// into the provided memory addresses using `MUL VL` addressing for the
/// currently active vector length.
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "sve")]
unsafe fn save_sve_regs(addr_z: u64, addr_p: u64, addr_ffr: u64, save_ffr: bool) {
    unsafe {
        // Save the z register bank
        asm!(
            "str z0, [{addr_z}, #0, MUL VL]",
            "str z1, [{addr_z}, #1, MUL VL]",
            "str z2, [{addr_z}, #2, MUL VL]",
            "str z3, [{addr_z}, #3, MUL VL]",
            "str z4, [{addr_z}, #4, MUL VL]",
            "str z5, [{addr_z}, #5, MUL VL]",
            "str z6, [{addr_z}, #6, MUL VL]",
            "str z7, [{addr_z}, #7, MUL VL]",
            "str z8, [{addr_z}, #8, MUL VL]",
            "str z9, [{addr_z}, #9, MUL VL]",
            "str z10, [{addr_z}, #10, MUL VL]",
            "str z11, [{addr_z}, #11, MUL VL]",
            "str z12, [{addr_z}, #12, MUL VL]",
            "str z13, [{addr_z}, #13, MUL VL]",
            "str z14, [{addr_z}, #14, MUL VL]",
            "str z15, [{addr_z}, #15, MUL VL]",
            "str z16, [{addr_z}, #16, MUL VL]",
            "str z17, [{addr_z}, #17, MUL VL]",
            "str z18, [{addr_z}, #18, MUL VL]",
            "str z19, [{addr_z}, #19, MUL VL]",
            "str z20, [{addr_z}, #20, MUL VL]",
            "str z21, [{addr_z}, #21, MUL VL]",
            "str z22, [{addr_z}, #22, MUL VL]",
            "str z23, [{addr_z}, #23, MUL VL]",
            "str z24, [{addr_z}, #24, MUL VL]",
            "str z25, [{addr_z}, #25, MUL VL]",
            "str z26, [{addr_z}, #26, MUL VL]",
            "str z27, [{addr_z}, #27, MUL VL]",
            "str z28, [{addr_z}, #28, MUL VL]",
            "str z29, [{addr_z}, #29, MUL VL]",
            "str z30, [{addr_z}, #30, MUL VL]",
            "str z31, [{addr_z}, #31, MUL VL]",
            addr_z = in(reg) addr_z,
        );
        // Save the p register bank
        asm!(
            "str p0, [{addr_p}, #0, MUL VL]",
            "str p1, [{addr_p}, #1, MUL VL]",
            "str p2, [{addr_p}, #2, MUL VL]",
            "str p3, [{addr_p}, #3, MUL VL]",
            "str p4, [{addr_p}, #4, MUL VL]",
            "str p5, [{addr_p}, #5, MUL VL]",
            "str p6, [{addr_p}, #6, MUL VL]",
            "str p7, [{addr_p}, #7, MUL VL]",
            "str p8, [{addr_p}, #8, MUL VL]",
            "str p9, [{addr_p}, #9, MUL VL]",
            "str p10, [{addr_p}, #10, MUL VL]",
            "str p11, [{addr_p}, #11, MUL VL]",
            "str p12, [{addr_p}, #12, MUL VL]",
            "str p13, [{addr_p}, #13, MUL VL]",
            "str p14, [{addr_p}, #14, MUL VL]",
            "str p15, [{addr_p}, #15, MUL VL]",
            addr_p = in(reg) addr_p,
        );
        if save_ffr {
            asm!(
                "rdffr p0.B",
                "str p0, [{addr_ffr}]",
                addr_ffr = in(reg) addr_ffr,
            );
        }
    }
}

/// # Safety
///
/// Restore the SVE register banks (`Z0..Z31`, `P0..P15`, and optionally `FFR`)
/// from the provided memory addresses using `MUL VL` addressing for the
/// currently active vector length.
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "sve")]
unsafe fn restore_sve_regs(addr_z: u64, addr_p: u64, addr_ffr: u64, restore_ffr: bool) {
    unsafe {
        // Restore the z register bank
        asm!(
            "ldr z0, [{addr_z}, #0, MUL VL]",
            "ldr z1, [{addr_z}, #1, MUL VL]",
            "ldr z2, [{addr_z}, #2, MUL VL]",
            "ldr z3, [{addr_z}, #3, MUL VL]",
            "ldr z4, [{addr_z}, #4, MUL VL]",
            "ldr z5, [{addr_z}, #5, MUL VL]",
            "ldr z6, [{addr_z}, #6, MUL VL]",
            "ldr z7, [{addr_z}, #7, MUL VL]",
            "ldr z8, [{addr_z}, #8, MUL VL]",
            "ldr z9, [{addr_z}, #9, MUL VL]",
            "ldr z10, [{addr_z}, #10, MUL VL]",
            "ldr z11, [{addr_z}, #11, MUL VL]",
            "ldr z12, [{addr_z}, #12, MUL VL]",
            "ldr z13, [{addr_z}, #13, MUL VL]",
            "ldr z14, [{addr_z}, #14, MUL VL]",
            "ldr z15, [{addr_z}, #15, MUL VL]",
            "ldr z16, [{addr_z}, #16, MUL VL]",
            "ldr z17, [{addr_z}, #17, MUL VL]",
            "ldr z18, [{addr_z}, #18, MUL VL]",
            "ldr z19, [{addr_z}, #19, MUL VL]",
            "ldr z20, [{addr_z}, #20, MUL VL]",
            "ldr z21, [{addr_z}, #21, MUL VL]",
            "ldr z22, [{addr_z}, #22, MUL VL]",
            "ldr z23, [{addr_z}, #23, MUL VL]",
            "ldr z24, [{addr_z}, #24, MUL VL]",
            "ldr z25, [{addr_z}, #25, MUL VL]",
            "ldr z26, [{addr_z}, #26, MUL VL]",
            "ldr z27, [{addr_z}, #27, MUL VL]",
            "ldr z28, [{addr_z}, #28, MUL VL]",
            "ldr z29, [{addr_z}, #29, MUL VL]",
            "ldr z30, [{addr_z}, #30, MUL VL]",
            "ldr z31, [{addr_z}, #31, MUL VL]",
            addr_z = in(reg) addr_z,
        );
        if restore_ffr {
            asm!(
                "ldr p0, [{addr_ffr}]",
                "wrffr p0.B",
                addr_ffr = in(reg) addr_ffr,
            );
        }
        // Restore the p register bank
        asm!(
            "ldr p0, [{addr_p}, #0, MUL VL]",
            "ldr p1, [{addr_p}, #1, MUL VL]",
            "ldr p2, [{addr_p}, #2, MUL VL]",
            "ldr p3, [{addr_p}, #3, MUL VL]",
            "ldr p4, [{addr_p}, #4, MUL VL]",
            "ldr p5, [{addr_p}, #5, MUL VL]",
            "ldr p6, [{addr_p}, #6, MUL VL]",
            "ldr p7, [{addr_p}, #7, MUL VL]",
            "ldr p8, [{addr_p}, #8, MUL VL]",
            "ldr p9, [{addr_p}, #9, MUL VL]",
            "ldr p10, [{addr_p}, #10, MUL VL]",
            "ldr p11, [{addr_p}, #11, MUL VL]",
            "ldr p12, [{addr_p}, #12, MUL VL]",
            "ldr p13, [{addr_p}, #13, MUL VL]",
            "ldr p14, [{addr_p}, #14, MUL VL]",
            "ldr p15, [{addr_p}, #15, MUL VL]",
            addr_p = in(reg) addr_p,
        );
    }
}

/// # Safety
///
/// Save the non-secure world's SVE register data (Z, P, FFR).
/// The storage is sized for the architectural maximum VL and is accessed
/// with a MUL VL stride for the currently active VL.
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "sve")]
unsafe fn save_ns_sve(sve: &mut NsSveRegs, save_ffr: bool) {
    let addr_z: u64 = sve.z.as_ptr() as u64;
    let addr_p: u64 = sve.p.as_ptr() as u64;
    let addr_ffr: u64 = sve.ffr.as_ptr() as u64;
    unsafe {
        save_sve_regs(addr_z, addr_p, addr_ffr, save_ffr);
    }
}

/// # Safety
///
/// Restore the non-secure world's SVE register data (Z, P, FFR).
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "sve")]
unsafe fn restore_ns_sve(sve: &NsSveRegs, restore_ffr: bool) {
    let addr_z: u64 = sve.z.as_ptr() as u64;
    let addr_p: u64 = sve.p.as_ptr() as u64;
    let addr_ffr: u64 = sve.ffr.as_ptr() as u64;
    unsafe {
        restore_sve_regs(addr_z, addr_p, addr_ffr, restore_ffr);
    }
}

/// # Safety
///
/// Save the REC's SVE register state into the per-REC SVE buffer, which is
/// backed by a contiguous multi-granule virtual address mapping.
/// The SVE register data is stored at the contiguous VA using VL-based addressing,
/// which works because the multi-granule mapping provides a contiguous address range.
/// The layout at `sve_va` is: Z registers (32 * VL bytes), then P registers (16 * VL/8 bytes),
/// then FFR (VL/8 bytes).
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "sve")]
unsafe fn save_rec_sve(sve_va: VirtAddr, save_ffr: bool) {
    let sve_va_usize = sve_va.as_usize();
    // P registers start right after the Z registers: NUM_VECTOR_REGS * VL
    // bytes after sve_va. The offset depends on the current vector length,
    // so read VL at runtime with rdvl; rec_sve_p_offset() keeps the save and
    // restore paths in agreement on the buffer layout.
    let vl_bytes: u64;
    unsafe {
        asm!("rdvl {vl}, #1", vl = out(reg) vl_bytes);
    }
    let addr_z: u64 = sve_va_usize as u64;
    let addr_p: u64 = (sve_va_usize + rec_sve_p_offset(vl_bytes as usize)) as u64;
    let addr_ffr: u64 = (sve_va_usize + rec_sve_ffr_offset(vl_bytes as usize)) as u64;
    unsafe {
        save_sve_regs(addr_z, addr_p, addr_ffr, save_ffr);
    }
}

/// # Safety
///
/// Restore the REC's SVE register state from the per-REC SVE buffer, which
/// is backed by a contiguous multi-granule virtual address mapping.
/// The layout at `sve_va` is: Z registers (32 * VL bytes), then P registers (16 * VL/8 bytes),
/// then FFR (VL/8 bytes).
#[allow(aarch64_softfloat_neon)]
#[inline(never)]
#[target_feature(enable = "sve")]
unsafe fn restore_rec_sve(sve_va: VirtAddr, restore_ffr: bool) {
    let sve_va_usize = sve_va.as_usize();
    // Compute VL and buffer offsets before loading Z0..Z31 so no Rust integer
    // computation executes after the SVE registers hold restored REC data.
    let vl_bytes: u64;
    unsafe {
        asm!("rdvl {vl}, #1", vl = out(reg) vl_bytes);
    }
    let addr_z: u64 = sve_va_usize as u64;
    let addr_p: u64 = (sve_va_usize + rec_sve_p_offset(vl_bytes as usize)) as u64;
    let addr_ffr: u64 = (sve_va_usize + rec_sve_ffr_offset(vl_bytes as usize)) as u64;

    unsafe {
        restore_sve_regs(addr_z, addr_p, addr_ffr, restore_ffr);
    }
}

// Returns true if the PE implements FEAT_SME and the given SVCR value indicates
// Streaming SVE mode (SVCR.SM == 1).
//
// In Armv9-A, FEAT_SME is architecturally independent of non-streaming FEAT_SVE:
// a PE may implement FEAT_SME without FEAT_SVE (`host_sve_en() == false` and
// `sme_en() == true`). When `SVCR.SM == 1` on any SME-capable PE, the full
// Streaming SVE register file (`Z0..Z31` of width `SVL`, `P0..P15`, and `FFR`
// when `FEAT_SME_FA64` is enabled) is active and must be saved/restored using
// SVE instructions rather than 128-bit FPU `Q0..Q31` loads/stores.
fn is_streaming(svcr: u64) -> bool {
    let svcr: InMemoryRegister<u64, SVCR::Register> = InMemoryRegister::new(svcr);
    sme_en() && svcr.read(SVCR::SM) != 0
}

// In Streaming SVE mode (`SVCR.SM == 1`), `rdffr`/`wrffr` are legal only if the
// full A64 instruction set is enabled in streaming mode (`SMCR_EL2.FA64 == 1`).
// In normal SVE mode (`SVCR.SM == 0`), `FFR` is always accessible.
fn preserve_ffr(svcr: u64) -> bool {
    if is_streaming(svcr) {
        SMCR_EL2.read(SMCR_EL2::FA64) != 0
    } else {
        true
    }
}

// This function is called when a SIMD access in Realm
// is made for the first time since REC_ENTER.
// See exception/trap.rs.
pub fn restore_state_lazy(rec: &Rec<'_>) {
    let rec_simd_ctx = &rec.context.simd;
    let rec_simd_regs = unsafe { rec_simd_ctx.sys_regs() };
    let mut ns_simd_ctx = NS_SIMD_CTX[get_cpu_id()].lock();

    // Disable simd traps during the context mgmt and synchronize before
    // accessing any SIMD/SVE/SME registers at EL2.
    CPTR_EL2.write(CPTR_EL2::TAM::SET);
    isb();

    // Capture the NS world's SVCR before saving NS registers (which checks
    // SVCR.SM via is_streaming() / preserve_ffr()) and before switching SVCR
    // to the REC's value.
    if sme_en() {
        ns_simd_ctx.sys_regs.svcr = SVCR.get();
    }

    // 1. Save the Non-Secure world's SIMD/SVE state.
    // The full SVE register set (Z0..Z31, P0..P15, FFR) must be saved instead
    // of just 128-bit Q0..Q31 in two cases:
    //   a) The PE supports non-streaming SVE (`host_sve_en()`): loading Q0..Q31
    //      for a non-SVE Realm zeroes the upper bits [VL_max-1:128] of Z0..Z31,
    //      so the full NS SVE width must be preserved even if this Realm has
    //      SVE disabled.
    //   b) The PE is an SME-only CPU (`host_sve_en() == false`, `sme_en() == true`)
    //      and the NS world entered RMM in Streaming SVE mode (`is_streaming(ns_simd_ctx.sys_regs.svcr)`):
    //      NS is actively using the streaming Z0..Z31, P0..P15, and FFR registers,
    //      which will be wiped by hardware as soon as SVCR.SM is cleared to 0
    //      for the Realm in step 2 below.
    if host_sve_en() || is_streaming(ns_simd_ctx.sys_regs.svcr) {
        // Guard ZCR_EL1 / ZCR_EL2 accesses with `host_sve_en()`: on an SME-only PE
        // (`host_sve_en() == false`), ZCR_EL1 and ZCR_EL2 are unimplemented and
        // accessing them raises an UNDEFINED exception at EL2. In Streaming SVE
        // mode, the active vector length (SVL) is controlled by SMCR_EL2.LEN
        // (which EL3 leaves untouched from NS-EL2), not by ZCR_EL2.LEN.
        if host_sve_en() {
            ns_simd_ctx.sys_regs.zcr_el12 = ZCR_EL1.get();
            ns_simd_ctx.sys_regs.zcr_el2 = ZCR_EL2.get();
            ZCR_EL2.write(ZCR_EL2::LEN.val(SVE_VQ_ARCH_MAX));
            isb();
        }
        unsafe {
            let save_ffr = preserve_ffr(ns_simd_ctx.sys_regs.svcr);
            save_ns_sve(&mut ns_simd_ctx.sve, save_ffr);
            save_fpu_crsr(&mut ns_simd_ctx.sys_regs);
        }
    } else {
        unsafe {
            save_fpu(&mut ns_simd_ctx.fpu);
            save_fpu_crsr(&mut ns_simd_ctx.sys_regs);
        }
    }

    // 2. Switch SVCR to the REC's value (0) after saving NS registers and
    // before restoring REC registers, since toggling SVCR.SM zeroes the
    // vector registers in hardware.
    if sme_en() {
        SVCR.set(rec_simd_regs.svcr);
        isb();
    }

    // 3. Restore the REC's SIMD/SVE state unconditionally (even when
    // !is_saved, as the zeroed AUX granules hold the REC's architectural
    // initial state, preventing NS register leakage into the Realm).
    if rec_simd_ctx.cfg.sve_en {
        // Install the REC's VL cap in ZCR_EL2 and the REC's ZCR_EL1 now
        // (after NS SVE state has been saved at SVE_VQ_ARCH_MAX) and
        // synchronize before restore_rec_sve() derives its MUL VL strides.
        ZCR_EL2.set(rec_simd_regs.zcr_el2);
        ZCR_EL1.set(rec_simd_regs.zcr_el12);
        isb();
        unsafe {
            let restore_ffr = true; // Since Realm is not supported with SME, it's always true.
            restore_rec_sve(rec_simd_ctx.fpu_sve_regs_va(), restore_ffr);
            restore_fpu_crsr(rec_simd_regs);
        }
        // Leave TFP=0 and TZ=0 for the Realm, but re-enable TSM=1 since SME
        // is not supported for Realms.
        CPTR_EL2.write(CPTR_EL2::TAM::SET + CPTR_EL2::TSM::SET);
    } else {
        unsafe {
            restore_fpu(rec_simd_ctx.fpu_regs());
            restore_fpu_crsr(rec_simd_regs);
        }
        // Leave TFP=0 for the Realm, but keep TZ=1 and TSM=1 so SVE and SME
        // instructions remain trapped for this non-SVE Realm.
        CPTR_EL2.write(CPTR_EL2::TAM::SET + CPTR_EL2::TSM::SET + CPTR_EL2::TZ::SET);
    }
    isb();
}

pub fn restore_state(_rec: &Rec<'_>) {
    // We don't save/restore any SIMD/SVE registers, ZCR_EL1/ZCR_EL2, or SVCR
    // until SIMD/SVE is actually accessed in the Realm. Setting TFP=1, TZ=1,
    // and TSM=1 below traps all FP/SVE/SME instructions as well as direct
    // accesses to ZCR_EL1/ZCR_EL2/SVCR into restore_state_lazy().
    // Note: As islet-rmm doesn't enable VHE in the realm world (HCR_EL2.E2H=0,
    // HCR_EL2.TGE=0), the non-E2H layout of CPTR_EL2 is used.
    CPTR_EL2
        .write(CPTR_EL2::TAM::SET + CPTR_EL2::TSM::SET + CPTR_EL2::TFP::SET + CPTR_EL2::TZ::SET);
    isb();
}

pub fn save_state(rec: &mut Rec<'_>) {
    let rec_simd_ctx = &mut rec.context.simd;

    // Disable SIMD traps for RMM context management (restoring RMM's default
    // CPTR_EL2 = TAM::SET configuration from setup_el2()).
    CPTR_EL2.write(CPTR_EL2::TAM::SET);
    isb();

    if !rec_simd_ctx.is_used {
        return;
    }
    // Since FEAT_SME is not for Realms, no need to store SVCR which doesn't change.

    let ns_simd_ctx = NS_SIMD_CTX[get_cpu_id()].lock();

    // 1. Save the REC's SIMD/SVE state.
    if rec_simd_ctx.cfg.sve_en {
        // ZCR_EL2 still holds rec_simd_regs.zcr_el2 here, so save_rec_sve() saves
        // at the REC's configured vector length.
        unsafe {
            rec_simd_ctx.sys_regs_mut().zcr_el12 = ZCR_EL1.get();
            let save_ffr = true; // Since FEAT_SME is not for Realms, it's always true.
            save_rec_sve(rec_simd_ctx.fpu_sve_regs_va(), save_ffr);
            save_fpu_crsr(rec_simd_ctx.sys_regs_mut());
        }
    } else {
        unsafe {
            save_fpu(rec_simd_ctx.fpu_regs_mut());
            save_fpu_crsr(rec_simd_ctx.sys_regs_mut());
        }
    }

    // 2. Restore NS SVCR before loading NS SIMD/SVE registers.
    // Otherwise, toggling SVCR.SM between 0 and 1 zeroes all SIMD registers.
    if sme_en() {
        SVCR.set(ns_simd_ctx.sys_regs.svcr);
        isb();
    }

    // 3. Restore the Non-Secure world's SIMD/SVE state.
    // Matches the condition in `restore_state_lazy()`: restore from `ns_simd_ctx.sve`
    // whenever the PE supports non-streaming SVE (`host_sve_en()`) or when
    // returning to NS in Streaming SVE mode (`is_streaming(ns_simd_ctx.sys_regs.svcr)`,
    // including on SME-only PEs where `host_sve_en() == false`).
    // Note that `SVCR` has already been restored in step 2 above, so if
    // `is_streaming(ns_simd_ctx.sys_regs.svcr)` is true, the PE is now in Streaming SVE
    // mode and `restore_ns_sve()` will reload the streaming Z/P/FFR registers.
    if host_sve_en() || is_streaming(ns_simd_ctx.sys_regs.svcr) {
        // Only touch ZCR_EL2 / ZCR_EL1 when non-streaming SVE is implemented
        // on the PE; on an SME-only PE (`host_sve_en() == false`), ZCR_ELx
        // registers do not exist and would trap as UNDEFINED.
        if host_sve_en() {
            // Restore NS SVE registers at SVE_VQ_ARCH_MAX so all implemented
            // vector bits are restored and any Realm state in upper bits is wiped.
            ZCR_EL2.write(ZCR_EL2::LEN.val(SVE_VQ_ARCH_MAX));
            isb();
        }
        unsafe {
            let restore_ffr = preserve_ffr(ns_simd_ctx.sys_regs.svcr);
            restore_ns_sve(&ns_simd_ctx.sve, restore_ffr);
            restore_fpu_crsr(&ns_simd_ctx.sys_regs);
        }
        if host_sve_en() {
            ZCR_EL2.set(ns_simd_ctx.sys_regs.zcr_el2);
            ZCR_EL1.set(ns_simd_ctx.sys_regs.zcr_el12);
            isb();
        }
    } else {
        unsafe {
            restore_fpu(&ns_simd_ctx.fpu);
            restore_fpu_crsr(&ns_simd_ctx.sys_regs);
        }
    }

    // To maintain immutability of rec.context during its restoration,
    // update the context here in advance.
    rec_simd_ctx.is_used = false;
    rec_simd_ctx.is_saved = true;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rmi::MAX_REC_AUX_GRANULES;

    #[test]
    fn test_sve_data_size_and_pages_needed() {
        // VL = (VQ + 1) * 128 bits
        // SVE data = 32 * (VQ+1) * 16 + 16 * (VQ+1) * 2 + (VQ+1) * 2 bytes
        assert_eq!(sve_data_size(0), 546); // VL = 128 bits
        assert_eq!(sve_data_size(4), 2730); // VL = 640 bits
        assert_eq!(sve_data_size(7), 4368); // VL = 1024 bits
        assert_eq!(sve_data_size(15), 8736); // VL = 2048 bits

        // Total number of 4KB AUX granules needed (SimdSysRegs header + SVE data)
        assert_eq!(sve_pages_needed(0), 1);
        assert_eq!(sve_pages_needed(4), 1);
        assert_eq!(sve_pages_needed(6), 1);
        assert_eq!(sve_pages_needed(7), 2);
        assert_eq!(sve_pages_needed(13), 2);
        assert_eq!(sve_pages_needed(15), 3);
    }

    #[test]
    fn test_aux_granule_layout_contract() {
        // The FPU/SVE register area (FpuRegs or SVE data) must start right
        // after the SimdSysRegs header in the contiguous AUX mapping, aligned
        // to FPU_SVE_REGS_ALIGN (16 bytes).
        assert_eq!(SimdContext::FPU_SVE_REGS_OFFSET, 0x30);
        assert!(SimdContext::FPU_SVE_REGS_OFFSET >= core::mem::size_of::<SimdSysRegs>());
        assert_eq!(SimdContext::FPU_SVE_REGS_OFFSET % FPU_SVE_REGS_ALIGN, 0);
        // When SVE is disabled, SimdSysRegs + FpuRegs must fit in 1 AUX granule.
        assert!(SimdContext::FPU_SVE_REGS_OFFSET + core::mem::size_of::<FpuRegs>() <= PAGE_SIZE);
    }

    #[test]
    fn test_rec_sve_offsets_and_bounds() {
        for vq in 0..=SVE_VQ_ARCH_MAX {
            let vl_bytes = (vq as usize + 1) * BYTES_PER_QUAD;
            let p_reg_size = vl_bytes / BITS_PER_BYTE;

            // Z0..Z31 occupy 32 * VL bytes at offset 0; P0..P15 start right after.
            let p_offset = rec_sve_p_offset(vl_bytes);
            assert_eq!(p_offset, 32 * vl_bytes);

            // P0..P15 occupy 16 * (VL / 8) = 2 * VL bytes; FFR starts right after.
            let ffr_offset = rec_sve_ffr_offset(vl_bytes);
            assert_eq!(ffr_offset, 34 * vl_bytes);
            assert_eq!(ffr_offset - p_offset, NUM_PREDICATE_REGS * p_reg_size);

            // End of FFR must match sve_data_size(vq) and fit in sve_pages_needed(vq) granules.
            let sve_end = ffr_offset + p_reg_size;
            assert_eq!(sve_end, sve_data_size(vq));
            assert!(SimdContext::FPU_SVE_REGS_OFFSET + sve_end <= sve_pages_needed(vq) * PAGE_SIZE);
        }
    }

    #[test]
    fn test_simd_context_accessors_and_offsets() {
        #[repr(C, align(16))]
        struct AlignedPage([u8; PAGE_SIZE]);

        let mut page = AlignedPage([0u8; PAGE_SIZE]);
        let base_addr = page.0.as_mut_ptr() as usize;
        let mut ctx = SimdContext {
            simd_va: Some(VirtAddr::from(base_addr)),
            ..Default::default()
        };

        assert_eq!(ctx.base_va(), base_addr);
        assert_eq!(
            ctx.fpu_sve_regs_va().as_usize(),
            base_addr + SimdContext::FPU_SVE_REGS_OFFSET
        );

        let sys_ptr = unsafe { ctx.sys_regs() as *const SimdSysRegs as usize };
        let fpu_ptr = unsafe { ctx.fpu_regs() as *const FpuRegs as usize };
        assert_eq!(sys_ptr, base_addr);
        assert_eq!(fpu_ptr, base_addr + SimdContext::FPU_SVE_REGS_OFFSET);
        assert!(sys_ptr + core::mem::size_of::<SimdSysRegs>() <= fpu_ptr);

        // Verify writes to SimdSysRegs and FpuRegs do not overlap.
        unsafe {
            let sys = ctx.sys_regs_mut();
            sys.fpcr = 0xDEAD_BEEF_CAFE_BABE;
            let fpu = ctx.fpu_regs_mut();
            fpu.q[0] = 0x1122_3344_5566_7788_99AA_BBCC_DDEE_FF00;

            assert_eq!(ctx.sys_regs().fpcr, 0xDEAD_BEEF_CAFE_BABE);
            assert_eq!(
                ctx.fpu_regs().q[0],
                0x1122_3344_5566_7788_99AA_BBCC_DDEE_FF00
            );
        }
    }

    #[test]
    fn test_simd_context_layout_with_aux_bump_allocator() {
        use crate::mm::aux_bump_allocator::AuxBumpAllocator;

        // 1. Non-SVE configuration
        let fpu_cfg = SimdConfig {
            sve_en: false,
            sve_vq: 0,
            sme_en: false,
        };
        let fpu_layout = simd_context_layout(&fpu_cfg).unwrap();
        assert_eq!(fpu_layout.size(), fpu_data_size());
        assert_eq!(fpu_layout.align(), FPU_SVE_REGS_ALIGN);

        let mut alloc = AuxBumpAllocator::new();
        let aux_bytes = aux_granules_needed(&fpu_cfg) * PAGE_SIZE;
        alloc.init(VirtAddr::from(0x2000_0000usize), aux_bytes);
        let ptr = alloc.alloc_layout(fpu_layout).unwrap();
        assert_eq!(ptr.as_ptr() as usize, 0x2000_0000);
        assert_eq!((ptr.as_ptr() as usize) % FPU_SVE_REGS_ALIGN, 0);
        assert_eq!(alloc.used(), fpu_data_size());

        // 2. All SVE VQ configurations fit and stay 16-byte aligned even after
        //    an unaligned 1-byte prior allocation in the bump allocator.
        for vq in 0..=SVE_VQ_ARCH_MAX {
            let sve_cfg = SimdConfig {
                sve_en: true,
                sve_vq: vq,
                sme_en: false,
            };
            let sve_layout = simd_context_layout(&sve_cfg).unwrap();
            assert_eq!(
                sve_layout.size(),
                SimdContext::FPU_SVE_REGS_OFFSET + sve_data_size(vq)
            );
            assert_eq!(sve_layout.align(), FPU_SVE_REGS_ALIGN);

            let total_aux_bytes = aux_granules_needed(&sve_cfg) * PAGE_SIZE;
            alloc.init(VirtAddr::from(0x2000_0000usize), total_aux_bytes);

            // Simulate a 1-byte allocation before init_simd() to verify alignment preservation
            let _ = alloc.alloc::<u8>().unwrap();
            let simd_ptr = alloc.alloc_layout(sve_layout).unwrap();
            let simd_va = simd_ptr.as_ptr() as usize;
            assert_eq!(simd_va % FPU_SVE_REGS_ALIGN, 0);
            assert_eq!(
                (simd_va + SimdContext::FPU_SVE_REGS_OFFSET) % FPU_SVE_REGS_ALIGN,
                0
            );
        }
    }

    #[test]
    fn test_ns_sve_regs_layout() {
        let max_vl_bytes = (SVE_VQ_ARCH_MAX as usize + 1) * BYTES_PER_QUAD;
        let ns_sve = NsSveRegs::default();

        assert_eq!(core::mem::align_of::<NsSveRegs>(), FPU_SVE_REGS_ALIGN);
        assert_eq!(
            core::mem::size_of_val(&ns_sve.z),
            rec_sve_p_offset(max_vl_bytes)
        );
        assert_eq!(
            core::mem::size_of_val(&ns_sve.p),
            rec_sve_ffr_offset(max_vl_bytes) - rec_sve_p_offset(max_vl_bytes)
        );
        assert_eq!(
            core::mem::size_of_val(&ns_sve.ffr),
            max_vl_bytes / BITS_PER_BYTE
        );
    }

    #[test]
    fn test_fpu_pages_needed() {
        // SimdSysRegs header (48 bytes) + FpuRegs (32 * 16 = 512 bytes) start
        // at offset 0 of the contiguous AUX mapping, so they always fit in a
        // single 4KB granule.
        assert_eq!(fpu_data_size(), SimdContext::FPU_SVE_REGS_OFFSET + 512);
        assert_eq!(fpu_pages_needed(), 1);
    }

    #[test]
    fn test_aux_granules_needed() {
        // SVE disabled: extra granule + the granule holding the SimdSysRegs
        // header and the FpuRegs bank.
        let mut cfg = SimdConfig {
            sve_en: false,
            sve_vq: 0,
            sme_en: false,
        };
        assert_eq!(
            aux_granules_needed(&cfg),
            EXTRA_AUX_GRANULES_COUNT + fpu_pages_needed()
        );

        // SVE enabled: extra granule + the granules holding the SimdSysRegs
        // header and the SVE register data.
        for vq in 0..=SVE_VQ_ARCH_MAX {
            cfg.sve_en = true;
            cfg.sve_vq = vq;
            assert_eq!(
                aux_granules_needed(&cfg),
                EXTRA_AUX_GRANULES_COUNT + sve_pages_needed(vq)
            );
        }
    }

    #[test]
    fn test_rec_aux_count_never_exceeds_max() {
        let mut cfg = SimdConfig {
            sve_en: false,
            sve_vq: 0,
            sme_en: false,
        };
        assert!(aux_granules_needed(&cfg) <= MAX_REC_AUX_GRANULES);

        cfg.sve_en = true;
        for vq in 0..=SVE_VQ_ARCH_MAX {
            cfg.sve_vq = vq;
            assert!(aux_granules_needed(&cfg) <= MAX_REC_AUX_GRANULES);
        }
    }
}
