use crate::{tools, GenericResult};
use clap::Args;
use colored::Colorize;

const CHALLENGE_LEN: u16 = 0x40;
const MAX_TOKEN_LEN: u16 = 0x1000;

pub(crate) fn version() -> GenericResult
{
    let version = rsi_el0::abi_version()?;
    println!("{}.{}", version.0, version.1);
    Ok(())
}

#[derive(Args, Debug)]
pub(crate) struct MeasurementReadArgs
{
    /// index to read, must be 0-4
    #[arg(short = 'n', long,
          value_parser = clap::value_parser!(u32).range(0..=4))]
    index: u32,

    /// filename to write the measurement, none for stdout hexdump
    #[arg(short, long)]
    output: Option<String>,
}

pub(crate) fn measurement_read(args: &MeasurementReadArgs) -> GenericResult
{
    let data = rsi_el0::measurement_read(args.index)?;

    match &args.output {
        Some(f) => tools::file_write(f, &data)?,
        None => println!("{:X?}", hex::encode(data)),
    }

    Ok(())
}

#[derive(Args, Debug)]
pub(crate) struct MeasurementExtendArgs
{
    /// index to extend, must be 1-4
    #[arg(short = 'n', long,
          value_parser = clap::value_parser!(u32).range(1..=4))]
    index: u32,

    /// length of random data to use (1-64)
    #[arg(short, long, default_value_t = rsi_el0::MAX_MEASUREMENT_LEN.into(),
          value_parser = clap::value_parser!(u32).range(1..=rsi_el0::MAX_MEASUREMENT_LEN.into()))]
    random: u32,

    /// filename to extend the measurement with (1-64 bytes), none to use random
    #[arg(short, long)]
    input: Option<String>,
}

pub(crate) fn measurement_extend(args: &MeasurementExtendArgs) -> GenericResult
{
    let data = match &args.input {
        None => tools::random_data(args.random as usize),
        Some(f) => tools::file_read(f)?,
    };

    if data.is_empty() || data.len() > rsi_el0::MAX_MEASUREMENT_LEN as usize {
        println!("Data must be within 1-64 bytes range");
        return Err(Box::new(nix::Error::E2BIG));
    }

    rsi_el0::measurement_extend(args.index, &data)?;

    Ok(())
}

#[derive(Args, Debug)]
pub(crate) struct AttestArgs
{
    /// filename with the challange (64 bytes), none to use random
    #[arg(short, long)]
    input: Option<String>,

    /// filename to write the token to, none to verify & print
    #[arg(short, long)]
    output: Option<String>,
}

pub(crate) fn attest(args: &AttestArgs) -> GenericResult
{
    let challenge = match &args.input {
        None => tools::random_data(rsi_el0::CHALLENGE_LEN as usize),
        Some(f) => tools::file_read(f)?,
    };

    if challenge.len() != rsi_el0::CHALLENGE_LEN as usize {
        println!("Challange needs to be exactly 64 bytes");
        return Err(Box::new(nix::Error::E2BIG));
    }

    // TODO: Error handling
    let token = &islet_sdk::attester::attest(&challenge).unwrap().buffer;

    match &args.output {
        None => tools::verify_print(token)?,
        Some(f) => tools::file_write(f, token)?,
    }

    Ok(())
}

#[derive(Args, Debug)]
pub(crate) struct VerifyArgs
{
    /// filename with the token to verify
    #[arg(short, long)]
    input: String,
}

pub(crate) fn verify(args: &VerifyArgs) -> GenericResult
{
    let token = tools::file_read(&args.input)?;
    tools::verify_print(&token)?;
    Ok(())
}

#[derive(Args, Debug)]
pub(crate) struct TestArgs
{
    /// filename with the token to verify
    #[arg(short, long)]
    verbose: bool,
}

pub(crate) fn test(args: &TestArgs) -> GenericResult
{
    match crate::checks::run_tests(args.verbose) {
        Ok(_) => (),
        Err(e) => {
            println!("{}: {}", "FAILED".red(), e);
            ()
        }
    }

    Ok(())
}

pub(crate) fn cloak_all() -> GenericResult
{
    let channel_id: usize = 0;
    let challenge: [u8; CHALLENGE_LEN as usize] = [0; CHALLENGE_LEN as usize];
    let write_data: [u8; MAX_TOKEN_LEN as usize] = [2; MAX_TOKEN_LEN as usize];
    let mut read_data: [u8; MAX_TOKEN_LEN as usize] = [0; MAX_TOKEN_LEN as usize];

    rsi_el0::cloak_create(channel_id)?;
    rsi_el0::cloak_connect(channel_id)?;

    let _data = rsi_el0::cloak_gen_report(channel_id, &challenge);
    //println!("report: {:X?}", hex::encode(data));
    rsi_el0::cloak_result(channel_id, 1)?;
    println!("cloak_all test - after cloak_result");

    rsi_el0::cloak_read(channel_id, &mut read_data)?;
    println!("first_read: {:x}", read_data[0]);

    rsi_el0::cloak_write(channel_id, &write_data)?;
    rsi_el0::cloak_read(channel_id, &mut read_data)?;
    println!("second_read: {:x}", read_data[0]);

    Ok(())
}

pub(crate) fn cloak_create() -> GenericResult
{
    let channel_id: usize = 0;
    rsi_el0::cloak_create(channel_id)?;
    Ok(())
}

pub(crate) fn cloak_connect() -> GenericResult
{
    let channel_id: usize = 0;
    rsi_el0::cloak_connect(channel_id)?;
    Ok(())
}

pub(crate) fn cloak_gen_report() -> GenericResult
{
    let channel_id: usize = 0;
    let challenge: [u8; CHALLENGE_LEN as usize] = [0; CHALLENGE_LEN as usize];

    let _data = rsi_el0::cloak_gen_report(channel_id, &challenge);
    Ok(())
}

pub(crate) fn cloak_result() -> GenericResult
{
    let channel_id: usize = 0;
    rsi_el0::cloak_result(channel_id, 1)?;
    Ok(())
}