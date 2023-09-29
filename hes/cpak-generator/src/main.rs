use clap::Parser;
use p384::pkcs8::{EncodePublicKey, LineEnding};
use std::fs::{self, File};
use std::io::{Read, Result as IOResult, Write};

/// Creates a path to a resource file
macro_rules! resource_file {
    ($fname:expr) => {
        // Ugly way to base path on workspace directory
        concat!(env!("CARGO_MANIFEST_DIR"), "/../res/", $fname)
    };
}

/// Creates a path to a default output dir
macro_rules! default_output_dir {
    () => {
        // Ugly way to base path on workspace directory
        concat!(env!("CARGO_MANIFEST_DIR"), "/../out/")
    };
}

/// Program for CPAK generation based on given BL2 hash and GUK
#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
#[command(
    about = "Program for CPAK generation in binary and PEM formats based on given BL2 hash and GUK"
)]
struct Args {
    /// Path to binary file with BL2 hash
    #[arg(short = 'b', long, value_name = "FILE")]
    #[arg(default_value = resource_file!("bl2_signed_hash.bin"))]
    hash_file: String,

    /// Path to binary file with GUK
    #[arg(short, long, value_name = "FILE")]
    #[arg(default_value = resource_file!("dummy_guk.bin"))]
    guk_file: String,

    /// Output directory to save CPAK files
    #[arg(short, long, value_name = "DIR")]
    #[arg(default_value = default_output_dir!())]
    output_dir: String,
}

fn load_binary_file(filename: &str) -> IOResult<Vec<u8>> {
    let mut f = File::open(filename)?;
    let metadata = fs::metadata(filename)?;
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer)?;

    Ok(buffer)
}

fn save_binary_file(filename: &str, data: &[u8]) -> IOResult<()> {
    let mut f = File::create(filename)?;
    f.write_all(data)
}

const PUBLIC_KEY_BIN: &str = "cpak_public.bin";
const PUBLIC_KEY_PEM: &str = "cpak_public.pem";

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let bl2_hash = load_binary_file(&args.hash_file)?;
    let guk = load_binary_file(&args.guk_file)?;

    fs::create_dir_all(args.output_dir.clone())?;

    const CPAK_SEED_LABEL: &[u8] = b"BL1_CPAK_SEED_DERIVATION";
    let seed = key_derivation::generate_seed(&bl2_hash, &guk, &CPAK_SEED_LABEL);

    let public_key = key_derivation::derive_p384_key(&seed, None).public_key();

    save_binary_file(
        &format!("{}/{}", args.output_dir.clone(), PUBLIC_KEY_BIN),
        &public_key.to_sec1_bytes(),
    )?;
    save_binary_file(
        &format!("{}/{}", args.output_dir.clone(), PUBLIC_KEY_PEM),
        &public_key
            .to_public_key_pem(LineEnding::LF)
            .unwrap()
            .as_bytes(),
    )?;

    Ok(())
}
