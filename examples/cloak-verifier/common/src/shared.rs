// Shared memory structs
pub const DEL_SYSCALL_OPEN: u32 = 1;
pub const DEL_OPEN_MAGIC_NUM: u32 = 9999999;

pub const DEL_SYSCALL_WRITE: u32 = 2;
pub const DEL_WRITE_MAGIC_NUM: u32 = 9999998;

pub const DEL_SYSCALL_READ: u32 = 3;
pub const DEL_READ_MAGIC_NUM: u32 = 9999997;

#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
pub struct SharedOpenReq
{
    pub dtype: u32,
    pub filename: [u8; 256],
    pub flags: u32,
    pub mode: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
pub struct SharedOpenResp
{
    pub dtype: u32,
    pub magic_num: u32,
    pub fd: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
pub struct SharedWriteReq
{
    pub dtype: u32,
    pub fd: i32,
    pub size: u32,
    pub data: [u8; 2048],
}

#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
pub struct SharedWriteResp
{
    pub dtype: u32,
    pub magic_num: u32,
    pub size: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
pub struct SharedReadReq
{
    pub dtype: u32,
    pub fd: i32,
    pub size: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
#[repr(packed)]
pub struct SharedReadResp
{
    pub dtype: u32,
    pub magic_num: u32,
    pub size: u32,
    pub data: [u8; 2048],
}

pub fn get_shared_type<T>(data: &[u8; 4096]) -> T
where
    T: Copy
{
    unsafe {
        let (head, body, _tail) = unsafe { data.align_to::<T>() };
        assert!(head.is_empty(), "Data was not aligned");
        let req = &body[0];
        *req
    }
}