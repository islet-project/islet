#[derive(Debug)]
pub enum Error {
    TypeMismatch,
    EndOfInput,
    Decoding,
    Format,
    CCAToken,
    CoseSign,
    RealmSignature,
    PlatformSignature,
    Claim(&'static str),
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
