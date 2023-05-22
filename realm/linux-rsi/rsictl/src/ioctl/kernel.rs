/*
 * This file must match kernel API.
 *
 * This includes rsi.h from the rsi module and eventually some internals from
 * the upstream kernel like the version split below.
 */

mod internal
{
    use super::{RsiMeasurement, RsiAttestation};

    nix::ioctl_read!(abi_version, b'x', 190u8, u32);
    nix::ioctl_readwrite_buf!(measurement_read, b'x', 192u8, RsiMeasurement);
    nix::ioctl_write_buf!(measurement_extend, b'x', 193u8, RsiMeasurement);
    nix::ioctl_readwrite_buf!(attestation_token, b'x', 194u8, RsiAttestation);
}


pub const MAX_MEASUR_LEN: u16 = 0x40;
pub const CHALLENGE_LEN:  u16 = 0x40;
pub const MAX_TOKEN_LEN:  u16 = 0x1000;


// should be pub(super) but nix leaks the type through pub ioctl definitions
#[repr(C)]
pub struct RsiMeasurement
{
    pub(super) index: u32,
    pub(super) data_len: u32,
    pub(super) data: [u8; MAX_MEASUR_LEN as usize],
}

impl RsiMeasurement
{
    pub(super) fn new_empty(index: u32) -> Self
    {
        Self { index, data_len: 0, data: [0; MAX_MEASUR_LEN as usize] }
    }

    pub(super) fn new_from_data(index: u32, src: &[u8]) -> Self
    {
        // panic on wrong size here to avoid obscured panic below
        assert!(!src.is_empty() && src.len() <= 64);

        let mut data = [0u8; MAX_MEASUR_LEN as usize];
        data[..src.len()].copy_from_slice(src);
        Self { index, data_len: src.len().try_into().unwrap(), data }
    }
}

// should be pub(super) but nix leaks the type through pub ioctl definitions
#[repr(C)]
pub struct RsiAttestation
{
    pub(super) challenge: [u8; CHALLENGE_LEN as usize],
    pub(super) token_len: u32,
    pub(super) token: [u8; MAX_TOKEN_LEN as usize],
}

impl RsiAttestation
{
    pub(super) fn new(src: &[u8; 64]) -> Self
    {
        Self { challenge: src.clone(), token_len: 0, token: [0; MAX_TOKEN_LEN as usize] }
    }
}

pub(super) const fn abi_version_get_major(version: u32) -> u32
{
    version >> 16
}

pub(super) const fn abi_version_get_minor(version: u32) -> u32
{
    version & 0xFFFF
}

pub(super) fn abi_version(fd: i32, data: *mut u32) -> nix::Result<()>
{
    unsafe { internal::abi_version(fd, data) }.map(|_| ())
}

pub(super) fn measurement_read(fd: i32, data: &mut [RsiMeasurement]) -> nix::Result<()>
{
    unsafe { internal::measurement_read(fd, data) }.map(|_| ())
}

pub(super) fn measurement_extend(fd: i32, data: &[RsiMeasurement]) -> nix::Result<()>
{
    unsafe { internal::measurement_extend(fd, data) }.map(|_| ())
}

pub(super) fn attestation_token(fd: i32, data: &mut [RsiAttestation]) -> nix::Result<()>
{
    unsafe { internal::attestation_token(fd, data) }.map(|_| ())
}
