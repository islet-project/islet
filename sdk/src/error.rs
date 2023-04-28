#[derive(Debug)]
pub enum Error {
    NotCCAToken,
    TypeMismatch,
    EndOfInput,
    Decoding,
    Format,
    PlatformToken(u16),
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
