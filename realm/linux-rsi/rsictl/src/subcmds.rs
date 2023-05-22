use crate::tools;
use clap::Args;


pub(crate) type GenericResult = Result<(), Box<dyn std::error::Error>>;

pub(crate) fn version() -> GenericResult
{
    let version = rsictl::abi_version()?;
    println!("{}.{}", version.0, version.1);
    Ok(())
}

#[derive(Args, Debug)]
pub(crate) struct MeasurReadArgs
{
    /// index to read, must be 0-4
    #[arg(short = 'n', long,
          value_parser = clap::value_parser!(u32).range(0..=4))]
    index: u32,

    /// filename to write the measurement, none for stdout hexdump
    #[arg(short, long)]
    output: Option<String>,
}

pub(crate) fn measur_read(args: &MeasurReadArgs) -> GenericResult
{
    let data = rsictl::measurement_read(args.index)?;

    match &args.output {
        Some(f) => tools::file_write(f, &data)?,
        None => tools::hexdump(&data, 8, None),
    }

    Ok(())
}

#[derive(Args, Debug)]
pub(crate) struct MeasurExtendArgs
{
    /// index to extend, must be 1-4
    #[arg(short = 'n', long,
          value_parser = clap::value_parser!(u32).range(1..=4))]
    index: u32,

    /// length of random data to use (1-64)
    #[arg(short, long, default_value_t = rsictl::MAX_MEASUR_LEN.into(),
          value_parser = clap::value_parser!(u32).range(1..=rsictl::MAX_MEASUR_LEN.into()))]
    random: u32,

    /// filename to extend the measurement with (1-64 bytes), none to use random
    #[arg(short, long)]
    input: Option<String>,
}

pub(crate) fn measur_extend(args: &MeasurExtendArgs) -> GenericResult
{
    let data = match &args.input {
        None => tools::random_data(args.random as usize),
        Some(f) => tools::file_read(f)?,
    };

    if data.is_empty() || data.len() > rsictl::MAX_MEASUR_LEN as usize {
        println!("Data must be within 1-64 bytes range");
        return Err(Box::new(nix::Error::E2BIG));
    }

    rsictl::measurement_extend(args.index, &data)?;

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
        None => tools::random_data(rsictl::CHALLENGE_LEN as usize),
        Some(f) => tools::file_read(f)?,
    };

    if challenge.len() != rsictl::CHALLENGE_LEN as usize {
        println!("Challange needs to be exactly 64 bytes");
        return Err(Box::new(nix::Error::E2BIG));
    }

    // try_into: &Vec<u8> -> &[u8,64]
    let token = rsictl::attestation_token(&challenge.try_into().unwrap())?;

    match &args.output {
        None => tools::verify_print(&token)?,
        Some(f) => tools::file_write(f, &token)?,
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
pub(crate) struct VerifyCArgs
{
    /// filename with the token to verify
    #[arg(short, long)]
    input: String,
}

pub(crate) fn verify_c(args: &VerifyCArgs) -> GenericResult
{
    let token = tools::file_read(&args.input)?;
    tools::verify_print_c(&token)?;
    Ok(())
}
