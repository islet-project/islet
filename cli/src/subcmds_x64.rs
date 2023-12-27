use crate::{tools, GenericResult};
use clap::Args;

use islet_sdk::prelude as sdk;

#[derive(Args, Debug)]
pub(crate) struct MeasurReadArgs
{
    /// index to read, must be 0-4 (0 for the RIM, 1 or greater for the REM)
    #[arg(short = 'n', long,
          value_parser = clap::value_parser!(u32).range(0..=4))]
    index: u32,
}

// TODO: Add error handling
pub(crate) fn measur_read(args: &MeasurReadArgs) -> GenericResult
{
    let report = sdk::attest(b"").expect("Failed to get an attestation report.");
    let claims = sdk::verify(&report).expect("Failed to verify the attestation report.");

    match &args.index {
        0 => {
            if let Some(sdk::ClaimData::Bstr(data)) =
                sdk::parse(&claims, sdk::config::STR_REALM_INITIAL_MEASUREMENT)
            {
                println!("{:X?}", hex::encode(data));
            } else {
                panic!("Failed to get the RIM.");
            }
        }
        _ => {
            panic!("REM is not supported yet.");
        }
    }

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
