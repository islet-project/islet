cfg_if::cfg_if! {
    if #[cfg(target_arch="aarch64")] {
        mod checks;
        mod subcmds_arm64;
        use subcmds_arm64 as subcmds;
    } else if #[cfg(target_arch="x86_64")] {
        mod subcmds_x64;
        use subcmds_x64 as subcmds;
    } else {
        unreachable!();
    }
}

mod tools;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli
{
    #[command(subcommand)]
    command: Commands,
}

cfg_if::cfg_if! {
    if #[cfg(target_arch="aarch64")] {
        #[derive(Subcommand, Debug)]
        enum Commands
        {
            /// Prints RSI ABI version
            Version,
            /// Gets given measurement
            MeasurementRead(subcmds::MeasurementReadArgs),
            /// Extends given measurement
            MeasurementExtend(subcmds::MeasurementExtendArgs),
            /// Gets attestation token
            Attest(subcmds::AttestArgs),
            /// Verifies and prints the token from a file
            Verify(subcmds::VerifyArgs),
            /// Verifies and prints the token from a file
            Test(subcmds::TestArgs),
            /// Cloak test all
            CloakAll,
            /// Cloak channel create
            CloakCreate,
            /// Cloak channel connect
            CloakConnect,
            /// Cloak channel gen_report
            CloakGenReport,
            /// Cloak channel result
            CloakResult,
        }
    } else if #[cfg(target_arch="x86_64")] {
        #[derive(Subcommand, Debug)]
        enum Commands
        {
            /// Gets given measurement
            MeasurementRead(subcmds::MeasurementReadArgs),
            /// Gets attestation token
            Attest(subcmds::AttestArgs),
            /// Verifies and prints the token from a file
            Verify(subcmds::VerifyArgs),
        }
    } else {
        unreachable!();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let cli = Cli::parse();

    cfg_if::cfg_if! {
        if #[cfg(target_arch="aarch64")] {
            match &cli.command {
                Commands::Version => subcmds::version()?,
                Commands::MeasurementRead(args) => subcmds::measurement_read(args)?,
                Commands::MeasurementExtend(args) => subcmds::measurement_extend(args)?,
                Commands::Attest(args) => subcmds::attest(args)?,
                Commands::Verify(args) => subcmds::verify(args)?,
                Commands::Test(args) => subcmds::test(args)?,
                Commands::CloakAll => subcmds::cloak_all()?,
                Commands::CloakCreate => subcmds::cloak_create()?,
                Commands::CloakConnect => subcmds::cloak_connect()?,
                Commands::CloakGenReport => subcmds::cloak_gen_report()?,
                Commands::CloakResult => subcmds::cloak_result()?,
            };
        } else if #[cfg(target_arch="x86_64")] {
            match &cli.command {
                Commands::MeasurementRead(args) => subcmds::measurement_read(args)?,
                Commands::Attest(args) => subcmds::attest(args)?,
                Commands::Verify(args) => subcmds::verify(args)?,
            }
        } else {
            unreachable!();
        }
    }

    Ok(())
}

pub(crate) type GenericResult = Result<(), Box<dyn std::error::Error>>;
