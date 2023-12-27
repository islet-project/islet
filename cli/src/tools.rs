use rand::{distributions::Standard, Rng};
use std::{
    fs::File,
    io::{Read, Write},
};

pub(crate) fn file_read(filename: &str) -> std::io::Result<Vec<u8>>
{
    let mut buf = Vec::<u8>::with_capacity(64);
    File::open(filename)?.read_to_end(&mut buf)?;
    buf.shrink_to_fit();
    Ok(buf)
}

pub(crate) fn file_write(filename: &str, data: &[u8]) -> std::io::Result<()>
{
    File::create(filename)?.write_all(data)
}

pub(crate) fn random_data(len: usize) -> Vec<u8>
{
    let rng = rand::thread_rng();
    rng.sample_iter(&Standard).take(len).collect()
}

pub(crate) fn verify_print(token: &[u8]) -> Result<(), cca_token::TokenError>
{
    let claims = cca_token::verifier::verify_token(token)?;
    cca_token::dumper::print_token(&claims);
    Ok(())
}
