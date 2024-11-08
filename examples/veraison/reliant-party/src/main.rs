use clap::Parser;
use realm_verifier::{
    parser_json::parse_value,
    RealmVerifier,
};
use std::{error::Error, fs::File, io::{Read, BufReader}, sync::Arc};

use ratls::{ChainVerifier, RaTlsServer};
use veraison_verifier::VeraisonTokenVerifer;

use log::{error, info};

/// Creates a path to a resource file
macro_rules! resource_file {
    ($fname:expr) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/res/", $fname)
    };
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// RaTls server bind address
    #[arg(short = 'b', long, default_value = "0.0.0.0:1337")]
    server_bind_address: String,

    /// JSON containing reference values
    #[arg(short, long, value_name = "FILE",
          default_value = resource_file!("example.json"))]
    reference_json: String,

    /// Path to server cert
    #[arg(short = 'c', long, value_name = "FILE",
          default_value = resource_file!("cert/server.crt"))]
    server_cert: String,

    /// Path to server private key
    #[arg(short = 'k', long, value_name = "FILE",
          default_value = resource_file!("cert/server.key"))]
    server_privkey: String,

    /// Veraison verification service host
    #[arg(short = 'v', long, default_value = "https://localhost:8080")]
    veraison_url: String,

    /// Veraisons public key to verify attestation results
    #[arg(short = 'p', long, default_value = resource_file!("keys/pkey.jwk"))]
    veraison_pubkey: String,


}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Cli::parse();

    let json_reader = BufReader::new(File::open(args.reference_json)?);
    let mut reference_json: serde_json::Value = serde_json::from_reader(json_reader)?;

    let reference_measurements = parse_value(reference_json["realm"]["reference-values"].take())?;

    let mut pubkey = String::new();
    let mut file = File::open(args.veraison_pubkey).map_err(|e| {
        error!("Failed to open veraison pubkey file: {:?}", e);
        e
    })?;
    file.read_to_string(&mut pubkey)?;

    let server = RaTlsServer::new(ratls::ServerMode::AttestedClient {
        client_token_verifier: Arc::new(ChainVerifier::new(vec![
            Arc::new(VeraisonTokenVerifer::new(args.veraison_url, pubkey, None).unwrap()),
            Arc::new(RealmVerifier::init(reference_measurements)),
        ])),
        server_certificate_path: args.server_cert,
        server_privatekey_path: args.server_privkey,
    })?;

    let mut conn_iter = server.connections(args.server_bind_address)?;
    loop {
        info!("Awaiting connection");
        let conn_iter_next = conn_iter.next();
        if conn_iter_next.is_none() {
            break;
        }
        match conn_iter_next.unwrap() {
            Ok(mut conn) => {
                info!("New connection accepted");
                let mut buf = Vec::new();
                buf.resize(0x100, 0u8);

                while let Ok(len) = conn.stream().read(&mut buf) {
                    info!(
                        "Message from client: {:?}",
                        String::from_utf8(buf[0..len].to_vec())?
                    );
                }

                info!("Connection closed");
            }
            Err(e) => {
                error!("Connection failed: {:?}", e);
            }
        }
    }

    Ok(())
}
