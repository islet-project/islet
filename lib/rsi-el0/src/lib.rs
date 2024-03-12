#![warn(rust_2018_idioms)]
#![deny(warnings)]

pub(crate) mod kernel;

use nix::{fcntl::OFlag, libc::O_RDWR, sys::stat::Mode};

pub const MAX_MEASUREMENT_LEN: u16 = 0x40;
pub const CHALLENGE_LEN: u16 = 0x40;
pub const MAX_TOKEN_LEN: u16 = 0x1000;

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
    Ok(measure[0].data[..(measure[0].data_len as usize)].to_vec())
}

pub fn measurement_extend(index: u32, data: &[u8]) -> nix::Result<()> {
    let measur = [kernel::RsiMeasurement::new_from_data(index, data)];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);
    kernel::measurement_extend(fd.get(), &measur)
}

pub fn attestation_token(challenge: &[u8; CHALLENGE_LEN as usize]) -> nix::Result<Vec<u8>> {
    let mut attest = [kernel::RsiAttestation::new(challenge)];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);
    kernel::attestation_token(fd.get(), &mut attest)?;
    Ok(attest[0].token[..(attest[0].token_len as usize)].to_vec())
}

pub fn cloak_create(id: usize) -> nix::Result<()> {
    let mut cloak = [kernel::RsiCloak::new()];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);

    cloak[0].id = id;
    kernel::cloak_create(fd.get(), &mut cloak)
}

pub fn cloak_connect(id: usize) -> nix::Result<()> {
    let mut cloak = [kernel::RsiCloak::new()];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);

    cloak[0].id = id;
    kernel::cloak_connect(fd.get(), &mut cloak)
}

pub fn cloak_gen_report(id: usize, challenge: &[u8; CHALLENGE_LEN as usize]) -> nix::Result<Vec<u8>> {
    let mut cloak = [kernel::RsiCloak::new()];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);

    cloak[0].id = id;
    cloak[0].challenge.copy_from_slice(challenge);
    kernel::cloak_gen_report(fd.get(), &mut cloak)?;
    Ok(cloak[0].token[..(cloak[0].token_len as usize)].to_vec())
}

pub fn cloak_result(id: usize, result: usize) -> nix::Result<()> {
    let mut cloak = [kernel::RsiCloak::new()];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);

    cloak[0].id = id;
    cloak[0].result = result;
    kernel::cloak_result(fd.get(), &mut cloak)
}

pub fn cloak_write(id: usize, data: &[u8; MAX_TOKEN_LEN as usize]) -> nix::Result<()> {
    let mut cloak = [kernel::RsiCloak::new()];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);

    cloak[0].id = id;
    cloak[0].token.copy_from_slice(data);
    kernel::cloak_write(fd.get(), &mut cloak)
}

pub fn cloak_read(id: usize, data: &mut [u8; MAX_TOKEN_LEN as usize]) -> nix::Result<()> {
    let mut cloak = [kernel::RsiCloak::new()];
    let fd = Fd::wrap(nix::fcntl::open(DEV, FLAGS, MODE)?);

    cloak[0].id = id;
    kernel::cloak_read(fd.get(), &mut cloak)?;
    data.copy_from_slice(&cloak[0].token);
    Ok(())
}
