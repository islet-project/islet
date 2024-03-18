extern crate serde;
extern crate serde_json;

use prusti_contracts::*;
use std::{net::TcpStream, sync::Arc, ops::DerefMut, ops::Deref};
use rustls::{ClientConnection, ClientConfig, server::DnsName, Stream, ConnectionCommon, SideData};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

use std::fs::File;
use std::io::{Read, BufReader, Write};
use common::ioctl::{measurement_extend, attestation_token};

//use log::{debug, info};
use rcgen::{CertificateParams, KeyPair, CustomExtension, Certificate as RcgenCert, date_time_ymd, DistinguishedName};
use rustls::{client::ResolvesClientCert, server::ResolvesServerCert, sign::{CertifiedKey, RsaSigningKey}, Certificate, PrivateKey, RootCertStore};
use rand::rngs::OsRng;
use pkcs8::{EncodePublicKey, EncodePrivateKey};
use crate::error::RaTlsError;
use base64::{Engine, engine::general_purpose::STANDARD as b64};
use sha2::{Sha512, Digest};
use lazy_static::lazy_static;
use simple_asn1::{OID, oid};

const CHALLENGE_LEN: u16 = 0x40;

#[trusted]
fn load_certificates_from_pem(path: &str) -> std::io::Result<Vec<Certificate>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)?;

    Ok(certs.into_iter().map(Certificate).collect())
}

#[trusted]
fn load_root_cert_store(path: impl AsRef<str>) -> Result<RootCertStore, RaTlsError> {
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

pub struct RaTlsConnection<C> {
    sock: TcpStream,
    conn: C,
}

impl<C: DerefMut + Deref<Target = ConnectionCommon<S>>, S: SideData> RaTlsConnection<C> {
    pub fn new(sock: TcpStream, conn: C) -> Self {
        Self { sock, conn }
    }

    #[trusted]
    pub fn stream<'a>(&'a mut self) -> Stream<'a, C, TcpStream> {
        Stream::new(&mut self.conn, &mut self.sock)
    }
}

pub enum ClientMode {
    AttestedClient {
        rem: Vec<u8>,
        root_ca_path: String
    }
}

pub struct RaTlsClient {
    mode: ClientMode
}

impl RaTlsClient {
    pub fn new(mode: ClientMode) -> Result<Self, RaTlsError> {
        Ok(Self { mode })
    }

    #[trusted]
    fn make_client_config(&self) -> Result<(ClientConfig, Option<String>), RaTlsError> {
        match &self.mode {
            ClientMode::AttestedClient { rem, root_ca_path } => {
                let mut rem_arr: [u8; 64] = [0; 64];
                for (dst, src) in rem_arr.iter_mut().zip(rem) {
                    *dst = *src;
                }

                Ok((ClientConfig::builder()
                        .with_safe_defaults()
                        .with_root_certificates(load_root_cert_store(root_ca_path)?)
                        .with_client_cert_resolver(Arc::new(RaTlsCertResolver::from_token_resolver(rem_arr)?)),
                    None
                ))
            }
        }
    }

    #[trusted]
    pub fn connect(&self, server_url: String, server_name: String) -> Result<RaTlsConnection<ClientConnection>, RaTlsError> {
        println!("connect start");
        let sock = TcpStream::connect(server_url)?;
        println!("TcpStream::connect start");
        let (config, challenge) = self.make_client_config()?;
        println!("make_client_config start");
        let conn = ClientConnection::new(
            Arc::new(config),
            rustls::ServerName::DnsName(DnsName::try_from(challenge.unwrap_or(server_name))?)
        )?;
        println!("ClientConnection::new start");

        let mut tlsconn = RaTlsConnection::new(sock, conn);
        println!("RaTlsConnection::new start");
        self.handshake(&mut tlsconn)?;

        println!("connect end");
        Ok(tlsconn)
    }

    #[trusted]
    fn handshake(&self, conn: &mut RaTlsConnection<ClientConnection>) -> Result<(), RaTlsError> {
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
            Err(RaTlsError::HandshakeError)
        }
    }
}

pub struct RaTlsCertResolver {
    //token_resolver: Arc<dyn InternalTokenResolver>,
    rem: [u8; 64],
    private_key: RsaPrivateKey
}

impl RaTlsCertResolver {
    #[trusted]
    pub fn from_token_resolver(rem: [u8; 64]) -> Result<Self, RaTlsError> {
        /*
        let key_size = 2048;

        info!("Generating RSA {}bit key.", key_size);
        let private_key = RsaPrivateKey::new(&mut OsRng, key_size)?;
        info!("Finished generating RSA key."); */

        if let Ok(mut file) = File::open("priv_key.serde.arm64") {
            let mut buf = vec![];
            if file.read_to_end(&mut buf).is_ok() {
                if let Ok(private_key) = serde_json::from_slice::<RsaPrivateKey>(&buf[..]) {
                    let pub_key = private_key.to_public_key();
                    let data = b"hello world";
                    let enc_data = pub_key.encrypt(&mut OsRng, Pkcs1v15Encrypt, &data[..]).expect("failed to encrypt");
                    let dec_data = private_key.decrypt(Pkcs1v15Encrypt, &enc_data).expect("failed to decrypt");
                    assert_eq!(&data[..], &dec_data[..]);

                    //info!("private_key read!");
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
    fn resolve(&self, challenge: &[u8]) -> Result<Vec<u8>, RaTlsError> {
        if challenge.len() != CHALLENGE_LEN as usize {
            return Err(RaTlsError::GenericTokenResolverError());
        }

        match measurement_extend(&self.rem)
        {
            Err(_) => {
                return Err(RaTlsError::GenericTokenResolverError());
            },
            Ok(_) => {},
        }

        match attestation_token(&challenge.try_into().unwrap())
        {
            Err(_) => Err(RaTlsError::GenericTokenResolverError()),
            Ok(v) => Ok(v),
        }
    }

    #[trusted]
    fn create_cert(&self, challenge: String) -> Result<Arc<CertifiedKey>, RaTlsError> {
        //info!("Received challenge {}", challenge);
        let realm_challenge = hash_realm_challenge(
            b64.decode(challenge)?.as_slice(),
            self.private_key
                .to_public_key()
                .to_public_key_der()?
                .as_bytes()
        );

        //info!("create_cert start");

        let token = self.resolve(&realm_challenge)?;
        let pkcs8_privkey = self.private_key.to_pkcs8_der()?;
        let privkey = PrivateKey(pkcs8_privkey.to_bytes().to_vec());
        let mut params = CertificateParams::default();
        params.key_pair = Some(KeyPair::try_from(pkcs8_privkey.as_bytes())?);
        params.not_before = date_time_ymd(2021, 05, 19);
        params.not_after = date_time_ymd(4096, 01, 01);
        params.distinguished_name = DistinguishedName::new();

        let cca_token_x509_ext = oid!(1, 3, 3, 3, 7);
        params.alg = &rcgen::PKCS_RSA_SHA256;
        params.custom_extensions.push(CustomExtension::from_oid_content(
            cca_token_x509_ext.as_vec::<u64>()?.as_slice(),
            token
        ));

        let der = RcgenCert::from_params(params)?.serialize_der()?;
        let cert = Certificate(der);
        let key = RsaSigningKey::new(&privkey)?;

        //info!("create_cert end");

        Ok(Arc::new(CertifiedKey {
            cert: vec![cert],
            key: Arc::new(key),
            ocsp: None,
            sct_list: None
        }))
    }
}

impl ResolvesClientCert for RaTlsCertResolver {
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

impl ResolvesServerCert for RaTlsCertResolver {
    #[trusted]
    fn resolve(&self, client_hello: rustls::server::ClientHello) -> Option<Arc<rustls::sign::CertifiedKey>> {
        if let Some(challenge) = client_hello.server_name() {
            self.create_cert(challenge.to_owned()).ok()
        } else {
            None
        }
    }
}