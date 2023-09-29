#![allow(non_camel_case_types)]

use islet_hes::{SW_TYPE_MAX_SIZE, VERSION_MAX_SIZE};

type psa_handle_t = i32;

pub const RSS_DELEGATED_SERVICE_HANDLE: psa_handle_t = 0x40000111;

pub const RSS_DELEGATED_ATTEST_GET_DELEGATED_KEY: i16 = 1001;
pub const RSS_DELEGATED_ATTEST_GET_PLATFORM_TOKEN: i16 = 1002;

pub const RSS_MEASURED_BOOT_SERVICE_HANDLE: psa_handle_t = 0x40000110;

pub const RSS_MEASURED_BOOT_READ: i16 = 1001;
pub const RSS_MEASURED_BOOT_EXTEND: i16 = 1002;

pub const PSA_MAX_IOVEC: usize = 4;
const PLAT_RSS_COMMS_PAYLOAD_MAX_SIZE: usize = 0x1000;

const TYPE_OFFSET: u8 = 16;
const TYPE_MASK: u32 = 0xFFFF << TYPE_OFFSET;
const IN_LEN_OFFSET: u8 = 8;
const IN_LEN_MASK: u32 = 0xFF << IN_LEN_OFFSET;
const OUT_LEN_OFFSET: u8 = 0;
const OUT_LEN_MASK: u32 = 0xFF << OUT_LEN_OFFSET;

pub const PSA_SUCCESS: i32 = 0;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct serialized_rss_comms_header_t {
    protocol_ver: u8,
    seq_num: u8,
    client_id: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct rss_embed_msg_t {
    handle: psa_handle_t,
    ctrl_param: u32, /* type, in_len, out_len */
    io_size: [u16; PSA_MAX_IOVEC],
    trailer: [u8; PLAT_RSS_COMMS_PAYLOAD_MAX_SIZE],
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
struct rss_embed_reply_t {
    return_val: i32,
    out_size: [u16; PSA_MAX_IOVEC],
    trailer: [u8; PLAT_RSS_COMMS_PAYLOAD_MAX_SIZE],
}

#[derive(Debug)]
#[repr(C, packed)]
struct serialized_rss_comms_msg_t {
    header: serialized_rss_comms_header_t,
    msg: rss_embed_msg_t,
}

#[derive(Debug)]
#[repr(C, packed)]
struct serialized_rss_comms_reply_t {
    header: serialized_rss_comms_header_t,
    reply: rss_embed_reply_t,
}

#[derive(Debug)]
#[repr(C)]
struct measured_boot_extend_iovec_t {
    index: u8,
    lock_measurement: u8,
    measurement_algo: u32,
    sw_type: [u8; SW_TYPE_MAX_SIZE],
    sw_type_size: u8,
}

#[derive(Debug)]
#[repr(C)]
struct measured_boot_read_iovec_in_t {
    index: u8,
    sw_type_size: u8,
    version_size: u8,
}

#[derive(Debug)]
#[repr(C)]
struct measured_boot_read_iovec_out_t {
    is_locked: u8,
    measurement_algo: u32,
    sw_type: [u8; SW_TYPE_MAX_SIZE],
    sw_type_len: u8,
    version: [u8; VERSION_MAX_SIZE],
    version_len: u8,
}

#[derive(Debug, Clone)]
pub(super) struct PSARequest {
    pub protocol_ver: u8,
    pub seq_num: u8,
    pub client_id: u16,
    pub handle: psa_handle_t,
    pub psa_type: i16,
    pub in_vecs: [Vec<u8>; PSA_MAX_IOVEC],
    pub out_lens: [usize; PSA_MAX_IOVEC],
}

pub enum PSAError {
    WrongDataLength,
}

impl PSARequest {
    pub(super) fn de(input: &[u8]) -> Result<Self, PSAError> {
        if input.len() > std::mem::size_of::<rss_embed_msg_t>() {
            return Err(PSAError::WrongDataLength);
        }

        let ptr = input.as_ptr() as *const serialized_rss_comms_msg_t;
        let msg: &serialized_rss_comms_msg_t = unsafe { &*ptr };

        let mut counter;

        let psa_type: i16 = ((msg.msg.ctrl_param & TYPE_MASK) >> TYPE_OFFSET)
            .try_into()
            .unwrap();
        let num_in_vecs: usize = ((msg.msg.ctrl_param & IN_LEN_MASK) >> IN_LEN_OFFSET)
            .try_into()
            .unwrap();
        let num_out_vecs: usize = ((msg.msg.ctrl_param & OUT_LEN_MASK) >> OUT_LEN_OFFSET)
            .try_into()
            .unwrap();

        /* required to copy on stack, msg is packed */
        let io_sizes = msg.msg.io_size;

        let mut in_vecs: [Vec<u8>; PSA_MAX_IOVEC] = Default::default();
        let mut offset = 0;
        counter = 0;
        for len in &io_sizes[..num_in_vecs] {
            let len = *len as usize;
            in_vecs[counter] = msg.msg.trailer[offset..offset + len].to_vec();
            counter = counter + 1;
            offset = offset + len;
        }

        let mut out_len = [0usize; PSA_MAX_IOVEC];
        counter = 0;
        for len in &io_sizes[num_in_vecs..num_in_vecs + num_out_vecs] {
            out_len[counter] = *len as usize;
            counter = counter + 1;
        }

        Ok(PSARequest {
            protocol_ver: msg.header.protocol_ver,
            seq_num: msg.header.seq_num,
            client_id: msg.header.client_id,
            handle: msg.msg.handle,
            psa_type,
            in_vecs,
            out_lens: out_len,
        })
    }
}

#[derive(Default, Debug)]
pub(super) struct PSAResponse {
    pub(super) protocol_ver: u8,
    pub(super) seq_num: u8,
    pub(super) client_id: u16,
    pub(super) return_val: i32,
    pub(super) out_vecs: [Vec<u8>; PSA_MAX_IOVEC],
}

impl PSAResponse {
    pub(super) fn ser(&self) -> Result<Vec<u8>, PSAError> {
        /* pack data */

        let header = serialized_rss_comms_header_t {
            protocol_ver: self.protocol_ver,
            seq_num: self.seq_num,
            client_id: self.client_id,
        };

        let mut out_size = [0u16; PSA_MAX_IOVEC];
        let mut trailer = [0u8; PLAT_RSS_COMMS_PAYLOAD_MAX_SIZE];
        let mut counter = 0;
        let mut offset = 0;
        for out_vec in &self.out_vecs {
            let len = out_vec.len();
            if len == 0 {
                break;
            }
            out_size[counter] = len.try_into().unwrap();

            let to_offset = offset + len;
            if to_offset >= PLAT_RSS_COMMS_PAYLOAD_MAX_SIZE {
                return Err(PSAError::WrongDataLength);
            }
            trailer[offset..to_offset].clone_from_slice(&out_vec);
            offset = to_offset;
            counter = counter + 1;
        }

        let reply = rss_embed_reply_t {
            return_val: self.return_val,
            out_size,
            trailer,
        };

        let res = serialized_rss_comms_reply_t { header, reply };

        /* convert to bytes */
        let length = std::mem::size_of::<serialized_rss_comms_reply_t>()
            - PLAT_RSS_COMMS_PAYLOAD_MAX_SIZE as usize
            + offset;

        let ptr = &res as *const serialized_rss_comms_reply_t as *const u8;
        let output = unsafe { std::slice::from_raw_parts(ptr, length) }.to_vec();

        Ok(output)
    }
}

#[derive(Debug)]
pub(super) struct ExtendRequest {
    pub(super) index: u8,
    pub(super) lock_measurement: u8,
    pub(super) measurement_algo: u32,
    pub(super) sw_type: [u8; SW_TYPE_MAX_SIZE],
    pub(super) sw_type_size: u8,
}

impl ExtendRequest {
    pub(super) fn de(input: &[u8]) -> Result<Self, PSAError> {
        if input.len() != std::mem::size_of::<measured_boot_extend_iovec_t>() {
            return Err(PSAError::WrongDataLength);
        }
        let ptr = input.as_ptr() as *const measured_boot_extend_iovec_t;
        let msg: &measured_boot_extend_iovec_t = unsafe { &*ptr };

        Ok(ExtendRequest {
            index: msg.index,
            lock_measurement: msg.lock_measurement,
            measurement_algo: msg.measurement_algo,
            sw_type: msg.sw_type,
            sw_type_size: msg.sw_type_size,
        })
    }
}

#[derive(Debug)]
pub struct ReadRequest {
    pub index: u8,
    pub sw_type_size: u8,
    pub version_size: u8,
}

impl ReadRequest {
    pub fn de(input: &[u8]) -> Result<Self, PSAError> {
        if input.len() != std::mem::size_of::<measured_boot_read_iovec_in_t>() {
            return Err(PSAError::WrongDataLength);
        }
        let ptr = input.as_ptr() as *const measured_boot_read_iovec_in_t;
        let msg: &measured_boot_read_iovec_in_t = unsafe { &*ptr };

        Ok(ReadRequest {
            index: msg.index,
            sw_type_size: msg.sw_type_size,
            version_size: msg.version_size,
        })
    }
}

pub struct ReadResponse {
    pub is_locked: u8,
    pub measurement_algo: u32,
    pub sw_type: [u8; SW_TYPE_MAX_SIZE],
    pub sw_type_len: u8,
    pub version: [u8; VERSION_MAX_SIZE],
    pub version_len: u8,
}

impl ReadResponse {
    pub(super) fn ser(&self) -> Result<Vec<u8>, PSAError> {
        /* pack data */

        let reply = measured_boot_read_iovec_out_t {
            is_locked: self.is_locked,
            measurement_algo: self.measurement_algo,
            sw_type: self.sw_type,
            sw_type_len: self.sw_type_len,
            version: self.version,
            version_len: self.version_len,
        };

        /* convert to bytes */
        let length = std::mem::size_of::<measured_boot_read_iovec_out_t>();

        let ptr = &reply as *const measured_boot_read_iovec_out_t as *const u8;
        let output = unsafe { std::slice::from_raw_parts(ptr, length) }.to_vec();

        Ok(output)
    }
}
