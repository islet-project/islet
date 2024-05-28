//extern crate alloc;
use crate::rmi::error::Error;
use core::ffi::CStr;
use crate::alloc::string::ToString;

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

    pub unsafe fn gpr(&self, idx: usize) -> u64 {
        if idx >= HOST_CALL_NR_GPRS {
            error!("out of index: {}", idx);
            return 0;
        }
        (*self.inner.val).gprs[idx]
    }

    // Safety: union type should be initialized
    // Check UB
    pub fn imm(&self) -> u16 {
        unsafe { self.inner.val.imm }
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

#[repr(C)]
struct _PrintInner {
    msg: [u8; 1024],
    data1: usize,
    data2: usize,
}

#[repr(C)]
union PrintInner {
    val: core::mem::ManuallyDrop<_PrintInner>,
    reserved: [u8; 2048],
}

#[repr(C)]
pub struct CloakPrintCall {
    inner: PrintInner,
}

impl CloakPrintCall {
    pub fn parse<'a>(addr: usize) -> &'a Self {
        unsafe { &*(addr as *const Self) }
    }

    pub fn print(&self) {
        let msg = unsafe { &self.inner.val.msg };
        let data1 = unsafe { self.inner.val.data1 };
        let data2 = unsafe { self.inner.val.data2 };

        let msg_str = match CStr::from_bytes_until_nul(msg) {
            Ok(v) => v.to_str().unwrap().to_string(),
            Err(_) => { return; }
        };

        info!("[RealmMsg] {}, {}-{}, {:X?}-{:X?}", msg_str, data1, data2, data1, data2);
    }
}