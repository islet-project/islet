use prusti_contracts::*;
use ioctl_gen::*;

use std::fs::{self, File};
use std::io;
use std::os::raw::{c_void, c_int, c_ulong};
use std::os::fd::{AsRawFd, RawFd};
use std::ptr;
use std::mem::size_of;

const MAX_MEASUR_LEN: u16 = 0x40;
const CHALLENGE_LEN: u16 = 0x40;
const MAX_TOKEN_LEN: u16 = 0x1000;
const DEV: &str = "/dev/rsi";

const MEASUREMENT_EXTEND: u32 = iow!(b'x', 193, size_of::<RsiMeasurement>());
const ATTESTATION_TOKEN: u32 = iowr!(b'x', 194, size_of::<RsiAttestation>());

const CHANNEL_CREATE: u32 = iowr!(b'x', 195, size_of::<RsiCloak>());
const CHANNEL_CONNECT: u32 = iowr!(b'x', 196, size_of::<RsiCloak>());
const CHANNEL_GEN_REPORT: u32 = iowr!(b'x', 197, size_of::<RsiCloak>());

const CHANNEL_WRITE: u32 = iowr!(b'x', 199, size_of::<RsiCloak>());
const CHANNEL_READ: u32 = iowr!(b'x', 200, size_of::<RsiCloak>());

extern "C" { fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int; }

#[repr(C)]
pub struct RsiMeasurement
{
    pub index: u32,
    pub data_len: u32,
    pub data: [u8; MAX_MEASUR_LEN as usize],
}

impl RsiMeasurement
{
    pub fn new_empty(index: u32) -> Self
    {
        Self { index, data_len: 0, data: [0; MAX_MEASUR_LEN as usize] }
    }

    #[trusted]
    pub fn new_from_data(index: u32, src: &[u8]) -> Self
    {
        // panic on wrong size here to avoid obscured panic below
        //assert!(!src.is_empty() && src.len() <= MAX_MEASUR_LEN as usize);

        let mut data = [0u8; MAX_MEASUR_LEN as usize];
        data.copy_from_slice(src);
        Self { index, data_len: src.len().try_into().unwrap(), data }
    }
}

#[repr(C)]
pub struct RsiAttestation
{
    pub challenge: [u8; CHALLENGE_LEN as usize],
    pub token_len: u32,
    pub token: [u8; MAX_TOKEN_LEN as usize],
}

impl RsiAttestation
{
    pub fn new(src: &[u8; CHALLENGE_LEN as usize]) -> Self
    {
        Self { challenge: src.clone(), token_len: 0, token: [0; MAX_TOKEN_LEN as usize] }
    }
}

#[repr(C)]
pub struct RsiCloak {
    id: usize,
    result: usize,
    challenge: [u8; CHALLENGE_LEN as usize],
    token_len: u32,
    token: [u8; MAX_TOKEN_LEN as usize],
}

impl RsiCloak {
    #[allow(dead_code)]
    pub fn new(id: usize) -> Self {
        Self {
            id: 0,
            result: 0,
            challenge: [0; CHALLENGE_LEN as usize],
            token_len: 0,
            token: [0; MAX_TOKEN_LEN as usize],
        }
    }
}

#[trusted]
fn convert_to_usize<T>(obj: &T) -> usize {
    obj as *const _ as usize
}

#[trusted]
fn ioctl_wrapper(fd: &File, request: usize, arg: usize) -> i32 {
    unsafe {
        ioctl(fd.as_raw_fd(), request as c_ulong, arg as c_ulong)
    }
}

pub fn cloak_create(id: usize) -> io::Result<()> {
    let fd = File::open(DEV)?;
    let cloak = RsiCloak::new(id);
    let cloak_addr = convert_to_usize(&cloak);

    match ioctl_wrapper(&fd, CHANNEL_CREATE as usize, cloak_addr) {
        0 => Ok(()),
        _ => Err(io::Error::from_raw_os_error(22)),
    }
}

pub fn cloak_connect(id: usize) -> io::Result<()> {
    let fd = File::open(DEV)?;
    let cloak = RsiCloak::new(id);
    let cloak_addr = convert_to_usize(&cloak);

    match ioctl_wrapper(&fd, CHANNEL_CONNECT as usize, cloak_addr) {
        0 => Ok(()),
        _ => Err(io::Error::from_raw_os_error(22)),
    }
}

pub fn cloak_gen_report(id: usize) -> io::Result<Vec<u8>> {
    let fd = File::open(DEV)?;
    let mut cloak = RsiCloak::new(id);
    let cloak_addr = convert_to_usize(&cloak);

    match ioctl_wrapper(&fd, CHANNEL_GEN_REPORT as usize, cloak_addr) {
        0 => Ok(cloak.token.to_vec()),
        _ => Err(io::Error::from_raw_os_error(22)),
    }
}

pub fn cloak_write(id: usize, data: &[u8; 4096]) -> io::Result<()> {
    let fd = File::open(DEV)?;
    let mut cloak = RsiCloak::new(id);
    cloak.token.copy_from_slice(data);
    let cloak_addr = convert_to_usize(&cloak);

    match ioctl_wrapper(&fd, CHANNEL_WRITE as usize, cloak_addr) {
        0 => Ok(()),
        _ => Err(io::Error::from_raw_os_error(22)),
    }
}

pub fn cloak_read(id: usize, data: &mut [u8; 4096]) -> io::Result<()> {
    let fd = File::open(DEV)?;
    let cloak = RsiCloak::new(id);
    let cloak_addr = convert_to_usize(&cloak);

    match ioctl_wrapper(&fd, CHANNEL_READ as usize, cloak_addr) {
        0 => {
            data.copy_from_slice(&cloak.token);
            Ok(())
        },
        _ => Err(io::Error::from_raw_os_error(22)),
    }
}

pub fn measurement_extend(data: &[u8]) -> io::Result<()> {
    let fd = File::open(DEV)?;
    let measure = RsiMeasurement::new_from_data(1, data);
    let measure_addr = convert_to_usize(&measure);

    match ioctl_wrapper(&fd, MEASUREMENT_EXTEND as usize, measure_addr) {
        0 => Ok(()),
        _ => Err(io::Error::from_raw_os_error(22)),
    }
}

pub fn attestation_token(challenge: &[u8; CHALLENGE_LEN as usize]) -> io::Result<Vec<u8>> {
    let fd = File::open(DEV)?;
    let mut att = RsiAttestation::new(challenge);
    let att_addr = convert_to_usize(&att);

    match ioctl_wrapper(&fd, ATTESTATION_TOKEN as usize, att_addr) {
        0 => Ok(att.token.to_vec()),
        _ => Err(io::Error::from_raw_os_error(22)),
    }
}
