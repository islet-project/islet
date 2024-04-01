use std::fs::File;
use std::io::{self, BufReader, Write};
use std::net::ToSocketAddrs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use argh::FromArgs;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime, SignatureVerificationAlgorithm};
use rustls_pemfile::{certs, pkcs8_private_keys};
//use rustls::client::{ServerCertVerifier, ServerCertVerified, ServerCertVerified};
use rustls::{RootCertStore, SignatureScheme, DistinguishedName, DigitallySignedStruct};
use rustls::server::danger::{ ClientCertVerifier, ClientCertVerified};
use rustls::server::ParsedCertificate;
use rustls::client::danger::HandshakeSignatureValid;
use rustls::client::verify_server_cert_signed_by_trust_anchor;
use tokio::io::{copy, sink, split, AsyncWriteExt, stdin as tokio_stdin, stdout as tokio_stdout};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{rustls, TlsAcceptor, TlsConnector};
use rcgen::Certificate;
use rand::RngCore;
use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;

pub static ALL_VERIFICATION_ALGS: &[&dyn SignatureVerificationAlgorithm] = &[
    #[cfg(feature = "ring")]
    ring::ECDSA_P256_SHA256,
    #[cfg(feature = "ring")]
    ring::ECDSA_P256_SHA384,
    #[cfg(feature = "ring")]
    ring::ECDSA_P384_SHA256,
    #[cfg(feature = "ring")]
    ring::ECDSA_P384_SHA384,
    #[cfg(feature = "ring")]
    ring::ED25519,
    #[cfg(all(feature = "ring", feature = "alloc"))]
    ring::RSA_PKCS1_2048_8192_SHA256,
    #[cfg(all(feature = "ring", feature = "alloc"))]
    ring::RSA_PKCS1_2048_8192_SHA384,
    #[cfg(all(feature = "ring", feature = "alloc"))]
    ring::RSA_PKCS1_2048_8192_SHA512,
    #[cfg(all(feature = "ring", feature = "alloc"))]
    ring::RSA_PKCS1_3072_8192_SHA384,
    #[cfg(all(feature = "ring", feature = "alloc"))]
    ring::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
    #[cfg(all(feature = "ring", feature = "alloc"))]
    ring::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
    #[cfg(all(feature = "ring", feature = "alloc"))]
    ring::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::ECDSA_P256_SHA256,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::ECDSA_P256_SHA384,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::ECDSA_P384_SHA256,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::ECDSA_P384_SHA384,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::ECDSA_P521_SHA512,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::ED25519,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::RSA_PKCS1_2048_8192_SHA256,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::RSA_PKCS1_2048_8192_SHA384,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::RSA_PKCS1_2048_8192_SHA512,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::RSA_PKCS1_3072_8192_SHA384,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
    #[cfg(feature = "aws_lc_rs")]
    aws_lc_rs::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
];

/*
struct SkipServerVerification;
impl SkipServerVerification {
    fn new() -> std::sync::Arc<Self> {
        Arc::new(Self)
    }
}

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

fn decode_spki_spk(spki_spk: &[u8]) -> Result<RsaPublicKey, InvalidSignature> {
    // public_key: unfortunately this is not a whole SPKI, but just the key material.
    // decode the two integers manually.
    let mut reader = der::SliceReader::new(spki_spk).map_err(|_| InvalidSignature)?;
    let ne: [der::asn1::UintRef; 2] = reader
        .decode()
        .map_err(|_| InvalidSignature)?;

    RsaPublicKey::new(
        BigUint::from_bytes_be(ne[0].as_bytes()),
        BigUint::from_bytes_be(ne[1].as_bytes()),
    )
    .map_err(|_| InvalidSignature)
} */

#[derive(Debug)]
struct CertVerif
{
    nonce: [u8; 64],
    root_subjects: Vec<DistinguishedName>,
    roots: RootCertStore,
}

impl CertVerif {
    fn new(roots: RootCertStore) -> Self {
        let mut buf = [0u8; 64];
        rand::thread_rng().fill_bytes(&mut buf);
        let root_subjects = vec![
            DistinguishedName::from(b64.encode(buf).as_bytes().to_owned())
        ];

        Self {
            nonce: buf,
            root_subjects: root_subjects,
            roots: roots,
        }
    }
}

impl ClientCertVerifier for CertVerif {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &self.root_subjects
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        now: UnixTime
    ) -> Result<ClientCertVerified, rustls::Error> {
        // check client cert
        println!("verify_client_cert!");
        let cert = ParsedCertificate::try_from(end_entity)?;

        println!("inter len: {}", intermediates.len());
        println!("after ParsedCertificate!");
        let res = verify_server_cert_signed_by_trust_anchor(&cert, &self.roots, intermediates, now, ALL_VERIFICATION_ALGS);
        if res.is_err() {
            println!("client cert verification failed.. skip.. <todo>");
        }

        Ok(ClientCertVerified::assertion())
        //Err(rustls::Error::UnsupportedNameType)
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        let mut v = Vec::new();
        v.push(SignatureScheme::RSA_PKCS1_SHA1);
        v.push(SignatureScheme::ECDSA_SHA1_Legacy);
        v.push(SignatureScheme::RSA_PKCS1_SHA256);
        v.push(SignatureScheme::ECDSA_NISTP256_SHA256);
        v.push(SignatureScheme::RSA_PKCS1_SHA384);
        v.push(SignatureScheme::ECDSA_NISTP384_SHA384);
        v.push(SignatureScheme::RSA_PKCS1_SHA512);
        v.push(SignatureScheme::ECDSA_NISTP521_SHA512);
        v.push(SignatureScheme::RSA_PSS_SHA256);
        v.push(SignatureScheme::RSA_PSS_SHA384);
        v.push(SignatureScheme::RSA_PSS_SHA512);
        v.push(SignatureScheme::ED25519);
        v.push(SignatureScheme::ED448);
        v
    }
}

fn add_root_cert_store(path: &Path, rc: &mut RootCertStore) {
    let mut reader = BufReader::new(File::open(path).expect("open error"));
    let c = certs(&mut reader);
    match c {
        Ok(v) => {
            for e in v.into_iter() {
                let _ = rc.add(CertificateDer::from(e));
            }
            println!("RootCertStore len: {}", rc.len());
        },
        Err(_) => panic!("certs error"),
    }
}

fn load_certs(path: &Path) -> Vec<CertificateDer> {
    let mut reader = BufReader::new(File::open(path).expect("open error"));
    //let c = certs(&mut reader)
    //    .collect::<Result<Vec<_>, _>>()
    //    .unwrap();
    let c = certs(&mut reader);
    match c {
        Ok(v) => {
            println!("cert_chain_len: {}", v.len());
            let mut cdv = Vec::new();
            for e in v.into_iter() {
                cdv.push(CertificateDer::from(e));
            }
            cdv
        },
        Err(_) => panic!("certs error!"),
    }
}

fn load_keys(path: &Path) -> PrivateKeyDer {
    let mut reader = BufReader::new(File::open(path).expect("open error"));
    let mut keys = pkcs8_private_keys(&mut reader).unwrap_or_else(|_| panic!("Failed to load keys"));
    println!("prv key len: {}", keys.len());

    let first_key = keys.remove(0);
    let k8 = PrivatePkcs8KeyDer::from(first_key);
    PrivateKeyDer::from(k8)
}

// server part
// root.crt - cvm1.crt
async fn run_server() -> io::Result<()> {
    let addr = "0.0.0.0:1888".to_socket_addrs()?.next().ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
    let certs = load_certs(Path::new("cvm1.crt"));
    let key = load_keys(Path::new("cvm1.key"));

    println!("load cert and key success");
    let mut root_cert_store = RootCertStore::empty();
    add_root_cert_store(Path::new("root.crt"), &mut root_cert_store);

    let verifier = CertVerif::new(root_cert_store);
    let config = rustls::ServerConfig::builder()
        //.with_no_client_auth()
        .with_client_cert_verifier(Arc::new(verifier))
        .with_single_cert(certs, key)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(&addr).await?;

    println!("bind success");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();

        println!("accept TCP success");

        let fut = async move {
            let stream = acceptor.accept(stream).await?;

            println!("accept TLS success!");

            let (mut reader, mut writer) = split(stream);
            let n = copy(&mut reader, &mut writer).await?;
            println!("after copy!");

            writer.flush().await?;
            println!("Echo: {} - {}", peer_addr, n);

            Ok(()) as io::Result<()>
        };
        
        tokio::spawn(async move {
            if let Err(err) = fut.await {
                eprintln!("{:?}", err);
            }
        });
    }
}

// client part
// root.crt - cvm2.crt
async fn run_client() -> io::Result<()> {
    let addr = "0.0.0.0:1888".to_socket_addrs()?.next().ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
    let domain = "localhost";
    let content = format!("GET / HTTP/1.0\r\nHost: {}\r\n\r\n", domain);
    
    let mut root_cert_store = RootCertStore::empty();
    add_root_cert_store(Path::new("root.crt"), &mut root_cert_store);

    let cert_chain = load_certs(Path::new("cvm2.crt"));
    let prv_key = load_keys(Path::new("cvm2.key"));

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_client_auth_cert(cert_chain, prv_key).expect("auth_cert");
        //.with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let stream = TcpStream::connect(&addr).await?;
    let (mut stdin, mut stdout) = (tokio_stdin(), tokio_stdout());
    let domain = ServerName::try_from(domain)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
        .to_owned();

    println!("[client] after TCP connect");

    let mut stream = connector.connect(domain, stream).await?;
    println!("[client] after TLS connect");

    stream.write_all(content.as_bytes()).await?;
    println!("[client] after write_all");

    let (mut reader, mut writer) = split(stream);
    tokio::select! {
        ret = copy(&mut reader, &mut stdout) => {
            ret?;
        },
        ret = copy(&mut stdin, &mut writer) => {
            ret?;
            writer.shutdown().await?
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    tokio::spawn(run_server());
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    println!("run_client: {:?}", run_client().await);
}