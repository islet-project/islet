mod resolver;

use clap::Parser;
use ratls::RaTlsClient;
use std::{io::Write, sync::Arc};


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
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    ratls::init_logger();

    let cli = Cli::parse();

    let client = RaTlsClient::new(ratls::ClientMode::AttestedClient {
        client_token_resolver: Arc::new(resolver::IoctlTokenResolver()),
        root_ca_path: cli.root_ca.to_string()
    })?;

    let mut connection = client.connect(cli.server_url.to_string(), cli.server_name.to_string())?;
    println!("Connection established");
    write!(connection.stream(), "GIT")?;
    println!("Work finished, exiting");

    Ok(())
}
