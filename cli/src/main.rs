mod checks;
mod subcmds;
mod token;
mod tools;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli
{
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands
{
    /// Prints RSI ABI version
    Version,
    /// Gets given measurement
    MeasurRead(subcmds::MeasurReadArgs),
    /// Extends given measurement
    MeasurExtend(subcmds::MeasurExtendArgs),
    /// Gets attestation token
    Attest(subcmds::AttestArgs),
    /// Verifies and prints the token from a file
    Verify(subcmds::VerifyArgs),
    /// Verifies and prints the token from a file
    Test(subcmds::TestArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let cli = Cli::parse();

    match &cli.command {
        Commands::Version => subcmds::version()?,
        Commands::MeasurRead(args) => subcmds::measur_read(args)?,
        Commands::MeasurExtend(args) => subcmds::measur_extend(args)?,
        Commands::Attest(args) => subcmds::attest(args)?,
        Commands::Verify(args) => subcmds::verify(args)?,
        Commands::Test(args) => subcmds::test(args)?,
    };

    Ok(())
}
