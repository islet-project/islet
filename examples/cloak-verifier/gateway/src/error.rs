use prusti_contracts::*;
use std::error::Error;
use std::fmt::Display;
use std::string::FromUtf8Error;
use base64::DecodeError;
use rustls::client::InvalidDnsNameError;
use rustls::sign::SignError;
use rustls;
use x509_certificate::X509CertificateError;
use rsa;

#[derive(Debug)]
pub enum RaTlsError {
    IOError(),
    RustlsError(),
    InvalidDnsName(),
    SinglePrivateKeyIsRequired,
    PrivateKeyParsingError(),
    InvalidCCAToken,
    RsaError(),
    Utf8DecodingError(),
    Base64DecodeError(),
    Pkcs8Error(),
    Pkcs8SpkiError(),
    RcgenError(),
    Asn1DecodeError(),
    Asn1EncodeError(),
    CertSignError(),
    CertParsingError(),
    MissingTokenInCertificate,
    CannotExtractTokenFromExtension,
    InvalidChallenge,
    HandshakeError,
    PkcsDERError(),

    GenericTokenResolverError(),
    GenericTokenVerifierError()
}

impl From<std::io::Error> for RaTlsError {
    #[trusted]
    fn from(_value: std::io::Error) -> Self {
        Self::IOError()
    }
}

impl From<rustls::Error> for RaTlsError {
    #[trusted]
    fn from(_value: rustls::Error) -> Self {
        Self::RustlsError()
    }
}

impl From<InvalidDnsNameError> for RaTlsError {
    #[trusted]
    fn from(_value: InvalidDnsNameError) -> Self {
        Self::InvalidDnsName()
    }
}

impl From<rsa::Error> for RaTlsError {
    #[trusted]
    fn from(_value: rsa::Error) -> Self {
        Self::RsaError()
    }
}

impl From<FromUtf8Error> for RaTlsError {
    #[trusted]
    fn from(_value: FromUtf8Error) -> Self {
        Self::Utf8DecodingError()
    }
}

impl From<DecodeError> for RaTlsError {
    #[trusted]
    fn from(_value: DecodeError) -> Self {
        Self::Base64DecodeError()
    }
}

impl From<pkcs8::Error> for RaTlsError {
    #[trusted]
    fn from(_value: pkcs8::Error) -> Self {
        Self::Pkcs8Error()
    }
}

impl From<pkcs8::spki::Error> for RaTlsError {
    #[trusted]
    fn from(_value: pkcs8::spki::Error) -> Self {
        Self::Pkcs8SpkiError()
    }
}

impl From<rcgen::Error> for RaTlsError {
    #[trusted]
    fn from(_value: rcgen::Error) -> Self {
        Self::RcgenError()
    }
}

impl From<simple_asn1::ASN1DecodeErr> for RaTlsError {
    #[trusted]
    fn from(_value: simple_asn1::ASN1DecodeErr) -> Self {
        Self::Asn1DecodeError()
    }
}

impl From<simple_asn1::ASN1EncodeErr> for RaTlsError {
    #[trusted]
    fn from(_value: simple_asn1::ASN1EncodeErr) -> Self {
        Self::Asn1EncodeError()
    }
}

impl From<SignError> for RaTlsError {
    #[trusted]
    fn from(_value: SignError) -> Self {
        Self::CertSignError()
    }
}

impl From<X509CertificateError> for RaTlsError {
    #[trusted]
    fn from(_value: X509CertificateError) -> Self {
        Self::CertParsingError()
    }
}

impl From<pkcs8::der::Error> for RaTlsError {
    #[trusted]
    fn from(_value: pkcs8::der::Error) -> Self {
        Self::PkcsDERError()
    }
}

impl Error for RaTlsError {}

impl Display for RaTlsError {
    #[trusted]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RaTlsError")
    }
}
