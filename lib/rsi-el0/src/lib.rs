#![warn(rust_2018_idioms)]
#![deny(warnings)]

pub(crate) mod kernel;

use nix::{fcntl::OFlag, libc::O_RDWR, sys::stat::Mode};
use nix::errno::Errno;

pub const MAX_MEASUREMENT_LEN: u16 = 0x40;
pub const CHALLENGE_LEN: u16 = 0x40;
pub const MAX_TOKEN_LEN: u16 = 0x1000;
pub const SEALING_KEY_LEN: usize = 32;
pub const RSI_SEALING_KEY_FLAGS_KEY:      u64 = 1 << 0;
pub const RSI_SEALING_KEY_FLAGS_RIM:      u64 = 1 << 1;
pub const RSI_SEALING_KEY_FLAGS_REALM_ID: u64 = 1 << 2;
pub const RSI_SEALING_KEY_FLAGS_SVN:      u64 = 1 << 3;

const FLAGS: OFlag = OFlag::from_bits_truncate(O_RDWR);
const MODE: Mode = Mode::from_bits_truncate(0o644);
const DEV: &str = "/dev/rsi";

struct Fd {
    fd: i32,
}

impl Fd {
    fn wrap(fd: i32) -> Self {
        Self { fd }
    }

    fn get(&self) -> i32 {
        self.fd
    }
}

impl Drop for Fd {
    fn drop(&mut self) {
        match nix::unistd::close(self.fd) {
            Ok(()) => (),
            Err(e) => println!("WARNING: close failed: {}", e),
        }
    }
}

pub fn abi_version() -> nix::Result<(u32, u32)> {
    let fd = Fd::wrap(nix::fcntl::open("/dev/rsi", FLAGS, MODE)?);
    let mut version = 0;
    kernel::abi_version(fd.get(), &mut version)?;
    Ok((
        kernel::abi_version_get_major(version),
        kernel::abi_version_get_minor(version),
    ))
}

pub fn measurement_read(index: u32) -> nix::Result<Vec<u8>> {
    let mut measure = [kernel::RsiMeasurement::new_empty(index)];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);
    kernel::measurement_read(fd.get(), &mut measure)?;
    // TODO: remove the below condition after linux-rsi module returns
    //       the correct measurement length
    if measure[0].data[32..] == [0; 32] {
        measure[0].data_len = 32;
    }
    Ok(measure[0].data[..(measure[0].data_len as usize)].to_vec())
}

pub fn measurement_extend(index: u32, data: &[u8]) -> nix::Result<()> {
    let measur = [kernel::RsiMeasurement::new_from_data(index, data)];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);
    kernel::measurement_extend(fd.get(), &measur)
}

// Use very small value to make sure the ERANGE case is tested.
// Optimally a value of 4096 should be used.
const INITIAL_TOKEN_SIZE: u64 = 64;

pub fn attestation_token(challenge: &[u8; CHALLENGE_LEN as usize]) -> nix::Result<Vec<u8>>
{
    let mut attest = [kernel::RsiAttestation::new(challenge, INITIAL_TOKEN_SIZE)];
    let mut token = vec![0 as u8; INITIAL_TOKEN_SIZE as usize];
    attest[0].token = token.as_mut_ptr();

    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);
    match kernel::attestation_token(fd.get(), &mut attest) {
        Ok(_) => (),
        Err(Errno::ERANGE) => {
            token = vec![0 as u8; attest[0].token_len as usize];
            attest[0].token = token.as_mut_ptr();
            kernel::attestation_token(fd.get(), &mut attest)?;
        },
        Err(e) => return Err(e),
    }
    Ok(token[..(attest[0].token_len as usize)].to_vec())
}

pub fn sealing_key(flags: u64, svn: u64) -> nix::Result<[u8; SEALING_KEY_LEN]>
{
    let mut sealing = [kernel::RsiSealingKey::new(flags, svn)];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);
    kernel::sealing_key(fd.get(), &mut sealing)?;
    Ok(sealing[0].realm_sealing_key)
}
