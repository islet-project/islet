use crate::host::Accessor as HostAccessor;

#[repr(C)]
pub struct Params {
    flags: Flags,
    mpidr: MPIDR,
    pc: PC,
    gprs: GPRs,
    inner: Inner,
}

impl Params {
    // Safety: union type should be initialized
    // Check UB
    pub fn pc(&self) -> usize {
        unsafe { self.pc.val as usize }
    }

    pub fn gprs(&self) -> &[u64] {
        unsafe { &self.gprs.val[..] }
    }
}

impl Default for Params {
    fn default() -> Self {
        Self {
            flags: Flags { val: 0 },
            mpidr: MPIDR { val: 0 },
            pc: PC { val: 0 },
            gprs: GPRs { val: [0; 8] },
            inner: Inner {
                val: core::mem::ManuallyDrop::new(_Inner {
                    num_aux: 0,
                    aux: [0; 16],
                }),
            },
        }
    }
}

impl Drop for Params {
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.inner.val);
        }
    }
}

impl HostAccessor for Params {}

impl core::fmt::Debug for Params {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Safety: union type should be initialized
        unsafe {
            f.debug_struct("rec::Params")
                .field("flags", &format_args!("{:#X}", &self.flags.val))
                .field("mpidr", &format_args!("{:#X}", &self.mpidr.val))
                .field("pc", &format_args!("{:#X}", &self.pc.val))
                .field("gprs", &self.gprs.val)
                .field("num_aux", &self.inner.val.num_aux)
                .field("aux", &self.inner.val.aux)
                .finish()
        }
    }
}

#[repr(C)]
union Flags {
    val: u64,
    reserved: [u8; 0x100],
}

#[repr(C)]
union MPIDR {
    val: u64,
    reserved: [u8; 0x200 - 0x100],
}

#[repr(C)]
union PC {
    val: u64,
    reserved: [u8; 0x300 - 0x200],
}

#[repr(C)]
union GPRs {
    val: [u64; 8],
    reserved: [u8; 0x800 - 0x300],
}

#[repr(C)]
struct _Inner {
    num_aux: u64,
    aux: [u64; 16],
}

#[repr(C)]
union Inner {
    val: core::mem::ManuallyDrop<_Inner>,
    reserved: [u8; 0x1000 - 0x800],
}
