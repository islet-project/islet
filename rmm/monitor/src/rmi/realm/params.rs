use crate::host::Accessor as HostAccessor;

#[repr(C)]
#[derive(Debug)]
pub struct Params {
    pub features_0: u64,
    padding0: [u8; 248],
    pub hash_algo: u8,
    padding1: [u8; 767],
    pub rpv: [u8; 64],
    padding2: [u8; 960],
    pub vmid: u16,
    padding3: [u8; 6],
    pub rtt_addr: u64,
    pub rtt_level_start: i64,
    pub rtt_num_start: u32,
    padding4: [u8; 2020],
}

impl Default for Params {
    fn default() -> Self {
        Self {
            features_0: 0,
            padding0: [0; 248],
            hash_algo: 0,
            padding1: [0; 767],
            rpv: [0; 64],
            padding2: [0; 960],
            vmid: 0,
            padding3: [0; 6],
            rtt_addr: 0,
            rtt_level_start: 0,
            rtt_num_start: 0,
            padding4: [0; 2020],
        }
    }
}

impl HostAccessor for Params {}

#[cfg(test)]
pub mod test {
    use super::*;

    macro_rules! offset_of {
        ($type:ty, $field:tt) => {{
            let dummy = core::mem::MaybeUninit::<$type>::uninit();
            let dummy_ptr = dummy.as_ptr();
            let member_ptr = unsafe { ::core::ptr::addr_of!((*dummy_ptr).$field) };

            member_ptr as usize - dummy_ptr as usize
        }};
    }

    #[test]
    fn spec_params() {
        assert_eq!(core::mem::size_of::<Params>(), 4096);

        assert_eq!(offset_of!(Params, features_0), 0x0);
        assert_eq!(offset_of!(Params, hash_algo), 0x100);
        assert_eq!(offset_of!(Params, rpv), 0x400);
        assert_eq!(offset_of!(Params, vmid), 0x800);
        assert_eq!(offset_of!(Params, rtt_addr), 0x808);
        assert_eq!(offset_of!(Params, rtt_level_start), 0x810);
        assert_eq!(offset_of!(Params, rtt_num_start), 0x818);
    }
}
