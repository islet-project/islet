use crate::host::Accessor as HostAccessor;

#[repr(C)]
pub struct Params {
    features0: Features0,
    hash_algo: HashAlgo,
    rpv: RPV,
    inner: Inner,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            features0: Features0 { val: 0 },
            hash_algo: HashAlgo { val: 0 },
            rpv: RPV { val: [0; RPV_SIZE] },
            inner: Inner {
                val: core::mem::ManuallyDrop::new(_Inner {
                    vmid: 0,
                    rtt_base: 0,
                    rtt_level_start: 0,
                    rtt_num_start: 0,
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
            f.debug_struct("Params")
                .field("features0", &format_args!("{:#X}", &self.features0.val))
                .field("hash_algo", &self.hash_algo.val)
                .field("rpv", &self.rpv.val)
                .field("vmid", &self.inner.val.vmid)
                .field("rtt_base", &format_args!("{:#X}", &self.inner.val.rtt_base))
                .field("rtt_level_start", &self.inner.val.rtt_level_start)
                .field("rtt_num_start", &self.inner.val.rtt_num_start)
                .finish()
        }
    }
}
#[repr(C)]
union Features0 {
    val: u64,
    reserved: [u8; 0x100],
}

#[repr(C)]
union HashAlgo {
    val: u8,
    reserved: [u8; 0x400 - 0x100],
}

const RPV_SIZE: usize = 64;
#[repr(C)]
union RPV {
    // Realm Personalization Value
    val: [u8; RPV_SIZE],
    reserved: [u8; 0x800 - 0x400],
}

#[repr(C)]
struct _Inner {
    vmid: u16,
    rtt_base: u64,
    rtt_level_start: i64,
    rtt_num_start: u32,
}

#[repr(C)]
union Inner {
    val: core::mem::ManuallyDrop<_Inner>,
    reserved: [u8; 0x1000 - 0x800],
}
