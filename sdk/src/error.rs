use cca_token::TokenError;

#[derive(Debug)]
pub enum Error {
    CCAToken(TokenError),
    Claims,
    Decoding,
    InvalidArgument,
    NotSupported,
    Report,
    Sealing,
    SealingKey,
    Serialize,
}

impl From<TokenError> for Error {
    fn from(err: TokenError) -> Self {
        Error::CCAToken(err)
    }
}
