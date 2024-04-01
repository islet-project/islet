extern crate serde;
extern crate serde_json;

use prusti_contracts::*;
use std::{net::TcpStream, sync::Arc, ops::DerefMut, ops::Deref, net::TcpListener};
use rustls::{ClientConnection, ClientConfig, server::DnsName, Stream, ConnectionCommon, SideData};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

use std::fs::File;
use std::time;
use std::io::{Read, BufReader, Write};
use std::process::Command;

//use log::{debug, info};
use rcgen::{CertificateParams, KeyPair, CustomExtension, Certificate as RcgenCert, date_time_ymd, DistinguishedName as RcgenDistinguishedName};
use rustls::{client::ResolvesClientCert, server::ClientCertVerified, server::ResolvesServerCert, sign::{CertifiedKey, RsaSigningKey}, Certificate, PrivateKey, RootCertStore};
use rustls::{ServerConnection, ServerConfig, server::ClientCertVerifier, DistinguishedName};
use rustls::client::{ServerCertVerifier, ServerCertVerified};
use rand::rngs::OsRng;
use pkcs8::{EncodePublicKey, EncodePrivateKey};
use crate::error::TlsError;
use base64::{Engine, engine::general_purpose::STANDARD as b64};
use sha2::{Sha512, Digest};
use simple_asn1::oid;
use rand::{self, RngCore};
use x509_certificate::X509Certificate;

use crate::error;

const CHALLENGE_LEN: u16 = 0x40;

// For RA-TLS
#[allow(dead_code)]
#[trusted]
fn load_certificates_from_pem(path: &str) -> std::io::Result<Vec<Certificate>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)?;

    Ok(certs.into_iter().map(Certificate).collect())
}

#[allow(dead_code)]
#[trusted]
fn load_private_key_from_file(path: &str) -> Result<PrivateKey, TlsError> {
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut reader)?;

    match keys.len() {
        0 => Err(TlsError::PrivateKeyParsingError()),
        1 => Ok(PrivateKey(keys.remove(0))),
        _ => Err(TlsError::PrivateKeyParsingError()),
    }
}

#[allow(dead_code)]
#[trusted]
fn load_root_cert_store(path: impl AsRef<str>) -> Result<RootCertStore, TlsError> {
    let root_ca = load_certificates_from_pem(path.as_ref())?;
    let mut root_store = RootCertStore::empty();

    for cert in root_ca.into_iter() {
        root_store.add(&cert)?;
    }

    Ok(root_store)
}

fn hash_realm_challenge(challenge: &[u8], der_public_key: &[u8]) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(challenge);
    hasher.update(der_public_key);
    hasher.finalize()[..].to_vec()
}

#[allow(dead_code)]
pub struct TlsConnection<C> {
    sock: TcpStream,
    conn: C,
}

impl<C: DerefMut + Deref<Target = ConnectionCommon<S>>, S: SideData> TlsConnection<C> {
    #[allow(dead_code)]
    pub fn new(sock: TcpStream, conn: C) -> Self {
        Self { sock, conn }
    }

    #[allow(dead_code)]
    #[trusted]
    pub fn stream<'a>(&'a mut self) -> Stream<'a, C, TcpStream> {
        Stream::new(&mut self.conn, &mut self.sock)
    }
}

#[allow(dead_code)]
pub enum ClientMode {
    TlsClient {
        rem: Vec<u8>,
        root_ca_path: String
    },
    CVMClient,
}

#[derive(Debug)]
struct CVMServerVerifier
{
    nonce: [u8; 64],
    root_subjects: Vec<DistinguishedName>,
    roots: RootCertStore,
    root_crt: Certificate,
}

impl CVMServerVerifier {
    fn new(roots: RootCertStore, root_crt: Certificate) -> Self {
        let mut buf = [0u8; 64];
        rand::thread_rng().fill_bytes(&mut buf);
        let root_subjects = vec![
            DistinguishedName::from(b64.encode(buf).as_bytes().to_owned())
        ];

        Self {
            nonce: buf,
            root_subjects: root_subjects,
            roots: roots,
            root_crt: root_crt,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct CVMServer;

impl CVMServer {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) -> Result<(), TlsError>{
        let certs = load_certificates_from_pem("cvm1.crt")?;
        let key = load_private_key_from_file("cvm1.key")?;
        let root_cert_store = load_root_cert_store("root.crt")?;
        let mut root_certs = load_certificates_from_pem("root.crt")?;
        let verifier = CVMServerVerifier::new(root_cert_store, root_certs.remove(0));

        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(Arc::new(verifier))
            .with_single_cert(certs, key)?;

        let conn = ServerConnection::new(Arc::new(config))?;
        let listener = TcpListener::bind("0.0.0.0:1888")?;
        let sock = listener.accept()?.0;
        let mut tlsconn = TlsConnection::new(sock, conn);

        println!("new connection accepted!");
        self.handshake(&mut tlsconn)?;
        println!("after handshake!");

        loop {
            let zero: [u8; 4096] = [0; 4096];
            let mut buf: [u8; 4096] = [0; 4096];

            while let Ok(len) = tlsconn.stream().read(&mut buf) {
                println!("message from client! {:x}", buf[0]);
            }
            buf.copy_from_slice(&zero);
        }
    }

    fn handshake(&self, conn: &mut TlsConnection<ServerConnection>) -> Result<(), TlsError> {
        let mut stream = conn.stream();
        let msg = "HELO";

        stream.write_all(msg.as_bytes())?;
        stream.flush()?;

        let mut resp = Vec::new();
        resp.resize(msg.len(), 0u8);
        stream.read_exact(&mut resp)?;

        if resp.as_slice() == msg.as_bytes() {
            Ok(())
        } else {
            Err(TlsError::HandshakeError)
        }
    }
}

impl ClientCertVerifier for CVMServerVerifier {
    fn client_auth_root_subjects(&self) -> &[DistinguishedName] {
        &self.root_subjects
    }

    fn verify_client_cert(
        &self,
        end_entity: &Certificate,
        _intermediates: &[Certificate],
        _now: time::SystemTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        println!("CVMServerVerifier: verify_client_cert!");

        // print cert info
        let end_cert = X509Certificate::from_der(end_entity.0.clone()).expect("x509_cert");
        let root_cert = X509Certificate::from_der((&self.root_crt).0.clone()).expect("x509_cert");
        println!("CVMServerVerifier: {:x?}", end_cert.serial_number_asn1().as_slice());

        // verification
        // (1) make a chain: end_entity + root.crt
        // (2) verify the chain
        let end_cert_pem = end_cert.encode_pem().expect("encode_pem");
        let root_cert_pem = root_cert.encode_pem().expect("encode_pem");

        let mut file = File::create("tmp_server_end_cert.pem").expect("file create");
        file.write_all(end_cert_pem.as_bytes()).expect("file write");

        let mut file = File::create("tmp_server_full_chain.pem").expect("file create");
        file.write_all(end_cert_pem.as_bytes()).expect("file write");
        file.write_all(root_cert_pem.as_bytes()).expect("file write");

        let output = Command::new("openssl")
            .arg("verify")
            .arg("-CAfile")
            .arg("tmp_server_full_chain.pem")
            .arg("tmp_server_end_cert.pem")
            .output()
            .expect("failed to execute process");

        let stdout_res = String::from_utf8_lossy(&output.stdout);
        let verification_res = stdout_res.find("OK");
        match verification_res {
            Some(_) => {
                println!("CVMServerVerifier: client_cert verification success!");
                Ok(ClientCertVerified::assertion())
            }
            None => {
                println!("CVMServerVerifier: client_cert verification fail!");
                Err(rustls::Error::UnsupportedNameType)
            },
        }
    }
}

pub struct CVMClientVerifier {
    root_crt: Certificate
}

impl CVMClientVerifier {
    pub fn new(root_crt: Certificate) -> Self {
        Self {
            root_crt: root_crt,
        }
    }
}

impl ServerCertVerifier for CVMClientVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: time::SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        println!("CVMClientVerifier: verify_server_cert()");

        // print cert info
        let end_cert = X509Certificate::from_der(end_entity.0.clone()).expect("x509_cert");
        let root_cert = X509Certificate::from_der((&self.root_crt).0.clone()).expect("x509_cert");
        println!("CVMClientVerifier: {:x?}", end_cert.serial_number_asn1().as_slice());

        // verification
        // (1) make a chain: end_entity + root.crt
        // (2) verify the chain
        let end_cert_pem = end_cert.encode_pem().expect("encode_pem");
        let root_cert_pem = root_cert.encode_pem().expect("encode_pem");

        let mut file = File::create("tmp_client_end_cert.pem").expect("file create");
        file.write_all(end_cert_pem.as_bytes()).expect("file write");

        let mut file = File::create("tmp_client_full_chain.pem").expect("file create");
        file.write_all(end_cert_pem.as_bytes()).expect("file write");
        file.write_all(root_cert_pem.as_bytes()).expect("file write");

        let output = Command::new("openssl")
            .arg("verify")
            .arg("-CAfile")
            .arg("tmp_client_full_chain.pem")
            .arg("tmp_client_end_cert.pem")
            .output()
            .expect("failed to execute process");

        let stdout_res = String::from_utf8_lossy(&output.stdout);
        let verification_res = stdout_res.find("OK");
        match verification_res {
            Some(_) => {
                println!("CVMClientVerifier: server_cert verification success!");
                Ok(ServerCertVerified::assertion())
            }
            None => {
                println!("CVMClientVerifier: server_cert verification fail!");
                Err(rustls::Error::UnsupportedNameType)
            },
        }

        //Ok(ServerCertVerified::assertion())
    }
}

#[allow(dead_code)]
pub struct TlsClient {
    mode: ClientMode
}

impl TlsClient {
    #[allow(dead_code)]
    pub fn new(mode: ClientMode) -> Result<Self, TlsError> {
        Ok(Self { mode })
    }

    #[allow(dead_code)]
    #[trusted]
    fn make_client_config(&self) -> Result<(ClientConfig, Option<String>), TlsError> {
        match &self.mode {
            ClientMode::TlsClient { rem, root_ca_path } => {
                let mut rem_arr: [u8; 64] = [0; 64];
                for (dst, src) in rem_arr.iter_mut().zip(rem) {
                    *dst = *src;
                }

                Ok((ClientConfig::builder()
                        .with_safe_defaults()
                        .with_root_certificates(load_root_cert_store(root_ca_path)?)
                        .with_client_cert_resolver(Arc::new(TlsCertResolver::from_token_resolver(rem_arr)?)),
                    None
                ))
            },
            ClientMode::CVMClient => {
                let mut root_certs = load_certificates_from_pem("root.crt").expect("load root crt");
                let verifier = CVMClientVerifier::new(root_certs.remove(0));

                Ok((ClientConfig::builder()
                        .with_safe_defaults()
                        //.with_root_certificates(load_root_cert_store("root.crt")?)
                        .with_custom_certificate_verifier(Arc::new(verifier))
                        .with_client_auth_cert(
                            load_certificates_from_pem("cvm2.crt")?,
                            load_private_key_from_file("cvm2.key")?,
                        )?,
                    None
                ))
            },
        }
    }

    #[allow(dead_code)]
    #[trusted]
    pub fn connect(&self, server_url: String, server_name: String) -> Result<TlsConnection<ClientConnection>, TlsError> {
        println!("connect start");
        let sock = TcpStream::connect(server_url)?;
        println!("TcpStream::connect start");
        let (config, challenge) = self.make_client_config()?;
        println!("make_client_config start");

        let conn = match &self.mode {
            ClientMode::TlsClient { rem, root_ca_path }=> {
                ClientConnection::new(
                    Arc::new(config),
                    rustls::ServerName::DnsName(DnsName::try_from(challenge.unwrap_or(server_name))?)
                )?
            },
            ClientMode::CVMClient => {
                ClientConnection::new(
                    Arc::new(config),
                    rustls::ServerName::DnsName(DnsName::try_from(server_name)?)
                )?
            }
        };
        println!("ClientConnection::new start");

        let mut tlsconn = TlsConnection::new(sock, conn);
        println!("TlsConnection::new start");
        self.handshake(&mut tlsconn)?;  // what does handshake() do?

        println!("connect end");
        Ok(tlsconn)
    }

    #[allow(dead_code)]
    #[trusted]
    fn handshake(&self, conn: &mut TlsConnection<ClientConnection>) -> Result<(), TlsError> {
        println!("handshake start");

        let mut stream = conn.stream();
        let msg = "HELO";

        let mut resp = Vec::new();
        resp.resize(msg.len(), 0u8);

        stream.read_exact(&mut resp)?;
        println!("read_exact start");

        stream.write_all(msg.as_bytes())?;
        stream.flush()?;
        println!("flush start");

        println!("handshake end");
        if resp.as_slice() == msg.as_bytes() {
            Ok(())
        } else {
            Err(TlsError::HandshakeError)
        }
    }
}

pub struct TlsCertResolver {
    //token_resolver: Arc<dyn InternalTokenResolver>,
    rem: [u8; 64],
    private_key: RsaPrivateKey
}

impl TlsCertResolver {
    #[trusted]
    pub fn from_token_resolver(rem: [u8; 64]) -> Result<Self, TlsError> {
        /*
        let key_size = 2048;

        info!("Generating RSA {}bit key.", key_size);
        let private_key = RsaPrivateKey::new(&mut OsRng, key_size)?;
        info!("Finished generating RSA key."); */

        // priv_key.serde.arm64 for RA-TLS with TTP
        if let Ok(mut file) = File::open("priv_key.serde.arm64") {
            let mut buf = vec![];
            if file.read_to_end(&mut buf).is_ok() {
                if let Ok(private_key) = serde_json::from_slice::<RsaPrivateKey>(&buf[..]) {
                    let pub_key = private_key.to_public_key();
                    let data = b"hello world";
                    //let enc_data = pub_key.encrypt(&mut OsRng, Pkcs1v15Encrypt, &data[..]).expect("failed to encrypt");
                    //let dec_data = private_key.decrypt(Pkcs1v15Encrypt, &enc_data).expect("failed to decrypt");
                    //assert_eq!(&data[..], &dec_data[..]);

                    println!("private_key read!");
                    return Ok(Self {
                        rem,
                        private_key,
                    });
                }
            }
        }

        let key_size = 2048;
        let private_key = RsaPrivateKey::new(&mut OsRng, key_size)?;
        //info!("Finished generating RSA key.");

        Ok(Self {
            rem,
            private_key
        })
    }

    #[trusted]
    fn resolve(&self, challenge: &[u8]) -> Result<Vec<u8>, TlsError> {
        Err(TlsError::GenericTokenResolverError())
        /*
        println!("resolve start!");
        if challenge.len() != CHALLENGE_LEN as usize {
            return Err(TlsError::GenericTokenResolverError());
        }
        println!("after challenge check!");

        match measurement_extend(&self.rem)
        {
            Err(_) => {
                return Err(TlsError::GenericTokenResolverError());
            },
            Ok(_) => {},
        }
        println!("after measurement_extend check!");

        match attestation_token(&challenge.try_into().unwrap())
        {
            Err(_) => Err(TlsError::GenericTokenResolverError()),
            Ok(v) => Ok(v),
        } */
    }

    #[trusted]
    fn create_cert(&self, challenge: String) -> Result<Arc<CertifiedKey>, TlsError> {
        println!("Received challenge {}", challenge);
        let realm_challenge = hash_realm_challenge(
            b64.decode(challenge)?.as_slice(),
            self.private_key
                .to_public_key()
                .to_public_key_der()?
                .as_bytes()
        );

        println!("create_cert start");

        let token = self.resolve(&realm_challenge)?;
        let pkcs8_privkey = self.private_key.to_pkcs8_der()?;
        let privkey = PrivateKey(pkcs8_privkey.to_bytes().to_vec());
        let mut params = CertificateParams::default();
        params.key_pair = Some(KeyPair::try_from(pkcs8_privkey.as_bytes())?);
        params.not_before = date_time_ymd(2021, 05, 19);
        params.not_after = date_time_ymd(4096, 01, 01);
        params.distinguished_name = RcgenDistinguishedName::new();

        let cca_token_x509_ext = oid!(1, 3, 3, 3, 7);
        params.alg = &rcgen::PKCS_RSA_SHA256;
        params.custom_extensions.push(CustomExtension::from_oid_content(
            cca_token_x509_ext.as_vec::<u64>()?.as_slice(),
            token
        ));

        let der = RcgenCert::from_params(params)?.serialize_der()?;
        let cert = Certificate(der);
        let key = RsaSigningKey::new(&privkey)?;

        println!("create_cert end");

        Ok(Arc::new(CertifiedKey {
            cert: vec![cert],
            key: Arc::new(key),
            ocsp: None,
            sct_list: None
        }))
    }
}

impl ResolvesClientCert for TlsCertResolver {
    fn has_certs(&self) -> bool {
        true
    }

    #[trusted]
    fn resolve(
            &self,
            acceptable_issuers: &[&[u8]],
            _sigschemes: &[rustls::SignatureScheme],
        ) -> Option<Arc<rustls::sign::CertifiedKey>> {

        if acceptable_issuers.len() != 1 {
            return None;
        }

        if let Ok(challenge) = String::from_utf8(acceptable_issuers[0].to_owned()) {
            self.create_cert(challenge).ok()
        } else {
            None
        }
    }
}

impl ResolvesServerCert for TlsCertResolver {
    #[trusted]
    fn resolve(&self, client_hello: rustls::server::ClientHello) -> Option<Arc<rustls::sign::CertifiedKey>> {
        if let Some(challenge) = client_hello.server_name() {
            self.create_cert(challenge.to_owned()).ok()
        } else {
            None
        }
    }
}

pub fn extract_cvm_pubkey(pubkey: &mut [u8; 4096]) -> bool {
    // RSA key pair for communication with other CVM
    if let Ok(mut file) = File::open("key_cvm1.serde") {
        let mut buf = vec![];
        if file.read_to_end(&mut buf).is_ok() {
            if let Ok(private_key) = serde_json::from_slice::<RsaPrivateKey>(&buf[..]) {
                let pub_key = private_key.to_public_key();
                let data = b"hello world";
                //let enc_data = pub_key.encrypt(&mut OsRng, Pkcs1v15Encrypt, &data[..]).expect("failed to encrypt");
                //let dec_data = private_key.decrypt(Pkcs1v15Encrypt, &enc_data).expect("failed to decrypt");
                //assert_eq!(&data[..], &dec_data[..]);
                println!("private_key read - key_cvm1!");

                let pub_key_vec = serde_json::to_vec(&pub_key);
                match pub_key_vec {
                    Ok(v) => {
                        println!("pub_key len: {}", v.len());
                        if v.len() > 4096 {
                            false
                        } else {
                            for (dst, src) in pubkey.iter_mut().zip(&v) {
                                *dst = *src;
                            }
                            true
                        }
                    },
                    Err(_) => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}

// For TLS between CVMs

