use std::fs::File;
use std::io::{self, BufReader, Write, Read};
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use std::thread;
use std::net::{TcpListener, TcpStream};
use std::time;

use argh::FromArgs;
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls::{self, ClientConnection, RootCertStore, SignatureScheme, DistinguishedName, DigitallySignedStruct, PrivateKey};
use rustls::server::{ ClientCertVerifier, ClientCertVerified};
use rustls::server::ParsedCertificate;
use rustls::client::HandshakeSignatureValid;
use rustls::client::verify_server_cert_signed_by_trust_anchor;
use rustls::{ServerConfig, ClientConfig};
use rand::RngCore;
use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use rustls::Certificate;

mod ttp;
mod error;

fn run_server() {
    let mut server = ttp::CVMServer::new();
    let _ = server.run();
}

fn run_client() {
    let client = ttp::TlsClient::new(ttp::ClientMode::CVMClient {});
    if client.is_err() {
        println!("ttp::TlsClient::new error");
        return;
    }
    println!("ttp::TlsClient::new success");

    let client = client.unwrap();
    let connection = client.connect("0.0.0.0:1888".to_string(), "localhost".to_string());
    if connection.is_err() {
        println!("client.connect error");
        return;
    }
    println!("client.connect success");

    let data: [u8; 4096] = [3; 4096];
    let mut connection = connection.unwrap();
    let write_res = connection.stream().write(&data);
    if write_res.is_err() {
        println!("connection.write error");
        return;
    }
    println!("connection.write success");
}

fn main() {
    let handle = thread::spawn(run_server);
    thread::sleep(time::Duration::from_secs(5));
    let _ = run_client();

    handle.join().unwrap();
}