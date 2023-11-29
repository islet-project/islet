use chrono::DateTime;
use clap::Parser;
use realm_verifier::{
    parser::{parse_reference_json, ReferenceJSON},
    MeasurementValue, RealmMeasurements, RealmVerifier,
};
use rust_rsi::CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS;
use std::{error::Error, fs::File, io::Read, sync::Arc};
use tinyvec::ArrayVec;

use ratls::{ChainVerifier, RaTlsServer};
use veraison_verifier::VeraisonTokenVerifer;

use log::{debug, error, info};

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
    #[arg(short = 'v', long, default_value = "http://localhost:8080")]
    veraison_url: String,

    /// Veraisons public key to verify attestation results
    #[arg(short = 'p', long, default_value = resource_file!("keys/pkey.jwk"))]
    veraison_pubkey: String,
}

fn hash_algo_len(hash_algo: String) -> Result<usize, &'static str> {
    match hash_algo.as_str() {
        "sha-256" => Ok(32),
        "sha-384" => Ok(48),
        "sha-512" => Ok(64),
        _ => Err("Invalid hash algorithm"),
    }
}

fn verify_reference_json(
    reference_json: ReferenceJSON,
) -> Result<RealmMeasurements, Box<dyn Error>> {
    // TODO: validate?
    let _release_timestamp = DateTime::parse_from_rfc3339(&reference_json.realm.release_timestamp)?;

    let hash_algo_len = hash_algo_len(reference_json.realm.reference_values.hash_algo)?;

    let rim: MeasurementValue = hex::decode(reference_json.realm.reference_values.rim).unwrap()
        [..hash_algo_len]
        .iter()
        .cloned()
        .collect();
    debug!("Reference RIM: {}", hex::encode(rim));
    let mut rems = Vec::new();
    for (rem_set_idx, string_rem_set) in reference_json
        .realm
        .reference_values
        .rems
        .iter()
        .enumerate()
    {
        if string_rem_set.len() != CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS {
            return Err("REM set len is invalid".into());
        }
        debug!("Reference REM set[{}]:", rem_set_idx);

        let mut rem_set = [ArrayVec::new(); CLAIM_COUNT_REALM_EXTENSIBLE_MEASUREMENTS];
        for (rem_idx, string_rem) in string_rem_set.iter().enumerate() {
            let rem: MeasurementValue = hex::decode(string_rem).unwrap().iter().cloned().collect();

            if rem.len() < hash_algo_len {
                return Err("Measurement len is too short for algorithm".into());
            }

            rem_set[rem_idx] = rem[0..hash_algo_len].iter().cloned().collect();
            debug!("REM[{}]: {:?}", rem_idx, hex::encode(rem_set[rem_idx]));
        }
        rems.push(rem_set);
    }

    Ok(RealmMeasurements {
        initial: rim,
        extensible: rems,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Cli::parse();

    let reference_json = parse_reference_json(args.reference_json).map_err(|e| {
        error!("Failed to parse reference json: {:?}", e);
        e
    })?;

    debug!("reference_json: {:#?}", reference_json);

    let realm_measurements = verify_reference_json(reference_json).map_err(|e| {
        error!("Reference json is not valid: {:?}", e);
        e
    })?;

    let mut pubkey = String::new();
    let mut file = File::open(args.veraison_pubkey).map_err(|e| {
        error!("Failed to open veraison pubkey file: {:?}", e);
        e
    })?;
    file.read_to_string(&mut pubkey)?;

    let server = RaTlsServer::new(ratls::ServerMode::AttestedClient {
        client_token_verifier: Arc::new(ChainVerifier::new(vec![
            Arc::new(VeraisonTokenVerifer::new(args.veraison_url, pubkey)),
            Arc::new(RealmVerifier::init(realm_measurements)),
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
