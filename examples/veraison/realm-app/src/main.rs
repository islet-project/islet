mod resolver;

use clap::Parser;
use ratls::RaTlsClient;
use std::{io::Write, sync::Arc, thread, time::Duration};

const MAX_TOKEN_LEN: u16 = 0x1000;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli
{
    /// Path to root CA cert
    #[arg(short, long)]
    root_ca: String,

    /// Url to ratls server
    #[arg(short = 'u', long, default_value = "localhost:1337")]
    server_url: String,

    /// Server name, overriden if server is attested
    #[clap(short = 'n', long, default_value = "localhost")]
    server_name: String,

    /// Cloak commands
    #[clap(short = 'c', long, default_value = "connect")]
    cloak_cmd: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    ratls::init_logger();

    let cli = Cli::parse();
    let channel_id = 0;
    let cloak_cmd = cli.cloak_cmd.to_string();

    if cloak_cmd == "connect" {
        match rust_rsi::cloak_connect(channel_id) {
            Ok(_) => {},
            Err(_) => {
                println!("cloak_connect error");
                return Ok(());
            },
        }
        println!("cloak_connect success");
    }
    else if cloak_cmd == "write" {
        let write_data: [u8; MAX_TOKEN_LEN as usize] = [2; MAX_TOKEN_LEN as usize];

        match rust_rsi::cloak_write(channel_id, &write_data) {
            Ok(_) => {},
            Err(_) => {
                println!("cloak_write error");
                return Ok(());
            },
        }
        println!("cloak_write success");
    }
    else if cloak_cmd == "read" {
        let mut read_data: [u8; MAX_TOKEN_LEN as usize] = [0; MAX_TOKEN_LEN as usize];

        match rust_rsi::cloak_read(channel_id, &mut read_data) {
            Ok(_) => {},
            Err(_) => {
                println!("cloak_read error");
                return Ok(());
            },
        }
        println!("cloak_read success: {:x}", read_data[0]);
    }

    /*
    let client = RaTlsClient::new(ratls::ClientMode::AttestedClient {
        client_token_resolver: Arc::new(resolver::IoctlTokenResolver()),
        root_ca_path: cli.root_ca.to_string()
    })?;

    let mut connection = client.connect(cli.server_url.to_string(), cli.server_name.to_string())?;
    println!("Connection established");
    write!(connection.stream(), "GIT")?;
    println!("Work finished, exiting"); */

    Ok(())
}
