//extern crate alloc;
use crate::rmi::error::Error;

#[repr(C)]
pub struct HostCall {
    inner: Inner,
}

impl HostCall {
    pub unsafe fn parse<'a>(addr: usize) -> &'a Self {
        &*(addr as *const Self)
    }

    pub unsafe fn parse_mut<'a>(addr: usize) -> &'a mut Self {
        &mut *(addr as *mut Self)
    }

    pub unsafe fn set_gpr(&mut self, idx: usize, val: u64) -> Result<(), Error> {
        if idx >= HOST_CALL_NR_GPRS {
            error!("out of index: {}", idx);
            return Err(Error::RmiErrorInput);
        }
        (*self.inner.val).gprs[idx] = val;
        Ok(())
    }

    // Safety: union type should be initialized
    // Check UB
    pub fn imm(&self) -> u16 {
        unsafe { self.inner.val.imm as u16 }
    }
}

impl Drop for HostCall {
    fn drop(&mut self) {
        unsafe {
            core::mem::ManuallyDrop::drop(&mut self.inner.val);
        }
    }
}

impl core::fmt::Debug for HostCall {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Safety: union type should be initialized
        unsafe {
            f.debug_struct("rsi::HostCall")
                .field("imm", &format_args!("{:#X}", &self.inner.val.imm))
                .field("gprs", &self.inner.val.gprs)
                .finish()
        }
    }
}

pub const HOST_CALL_NR_GPRS: usize = 7;

#[repr(C)]
struct _Inner {
    imm: u16,
    gprs: [u64; HOST_CALL_NR_GPRS],
}

#[repr(C)]
union Inner {
    val: core::mem::ManuallyDrop<_Inner>,
    reserved: [u8; 0x100],
}
