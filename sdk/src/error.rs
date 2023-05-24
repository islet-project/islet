#[derive(Debug)]
pub enum Error {
    AttestationReport,
    CCAToken,
    Claim(&'static str),
    ClaimCount,
    Claims,
    CoseSign,
    Decoding,
    EndOfInput,
    Format,
    InvalidArgument,
    NotSupported,
    PlatformSignature,
    RealmSignature,
    Report,
    Sealing,
    SealingKey,
    Serialize,
    TypeMismatch,
}

impl From<minicbor::decode::Error> for Error {
    fn from(err: minicbor::decode::Error) -> Self {
        if err.is_type_mismatch() {
            Error::TypeMismatch
        } else if err.is_end_of_input() {
            Error::EndOfInput
        } else {
            println!("Decoding error: {:?}", err);
            Error::Decoding
        }
    }
}
