mod resolver;

use clap::Parser;
use ratls::RaTlsClient;
use std::{io::Write, io::stdin, io::stdout, io::BufRead, sync::Arc, thread, time::Duration};
use sha2::{Digest, Sha512};
use rust_rsi;

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
    #[clap(short = 'c', long, default_value = "create")]
    cloak_cmd: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    ratls::init_logger();

    let cli = Cli::parse();
    let channel_id = 0;
    let cloak_cmd = cli.cloak_cmd.to_string();

    if cloak_cmd == "create" {
        match rust_rsi::cloak_create(channel_id) {
            Ok(_) => {},
            Err(_) => {
                println!("cloak_create error");
                return Ok(());
            },
        }
        println!("cloak_create success");
        return Ok(());
    }
    else if cloak_cmd != "gen_report" {
        println!("wrong cloak_cmd");
        return Ok(());
    }

    // 1-2. wait for CVM_app to connect to (polling) <TODO> notification?
    let mut counterpart_token: Vec<u8> = Vec::new();
    match rust_rsi::cloak_gen_report(channel_id) {
        Ok(token) => {
            counterpart_token.clone_from(&token);
        },
        Err(_) => {
            println!("cloak_gen_report error");
            return Ok(());
        },
    }
    let mut hasher = Sha512::new();
    hasher.update(&counterpart_token);
    let counterpart_hash = hasher.finalize()[..].to_vec();
    println!("cloak_gen_report success");

    // 2. remote channel with TTP
    // Gateway(RIM) + App(REM)
    let mut token_resolver = resolver::IoctlTokenResolver::new();
    for (dst, src) in token_resolver.user_data.iter_mut().zip(&counterpart_hash) {
        *dst = *src;
    }

    let client = RaTlsClient::new(ratls::ClientMode::AttestedClient {
        client_token_resolver: Arc::new(token_resolver),
        root_ca_path: cli.root_ca.to_string()
    })?;

    let mut connection = client.connect(cli.server_url.to_string(), cli.server_name.to_string())?;
    println!("Connection established");
    write!(connection.stream(), "GIT")?;
    println!("Work finished, exiting");

    // 3. local channel completed
    match rust_rsi::cloak_result(channel_id, 1) {
        Ok(_) => {},
        Err(_) => {
            println!("cloak_result error");
            return Ok(());
        },
    }
    println!("local channel established!");
    println!("remote channel with TTP established!");

    // 4. local channel: data exchange test
    println!("enter some text to read local channel data...");
    let mut line = String::new();
    let stdin = stdin();
    stdin.lock().read_line(&mut line).unwrap();

    let mut read_data: [u8; rust_rsi::MAX_TOKEN_LEN as usize] = [0; rust_rsi::MAX_TOKEN_LEN as usize];
    match rust_rsi::cloak_read(channel_id, &mut read_data) {
        Ok(_) => {},
        Err(_) => {
            println!("cloak_read error");
            return Ok(());
        },
    }
    println!("cloak_read success: {:x}", read_data[0]);

    // 5. remote channel with the other device
    // <TODO>
    Ok(())
}
