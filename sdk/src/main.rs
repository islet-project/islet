mod bootstrap;

use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};

#[derive(StructOpt)]
#[structopt(about = "Realm SDK")]
struct Args {
    input_file_path: String,
    output_file_path: String,
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();

    let mut input = File::open(args.input_file_path)
        .await
        .expect("Failed to open");

    let mut output = File::create(args.output_file_path)
        .await
        .expect("Failed to create");

    output
        .write_all(bootstrap::get_binary())
        .await
        .expect("Failed to write");

    io::copy(&mut input, &mut output)
        .await
        .expect("Failed to copy");
}
