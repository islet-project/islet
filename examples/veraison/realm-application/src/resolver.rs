use ratls::{InternalTokenResolver, RaTlsError};

pub struct IoctlTokenResolver();

impl InternalTokenResolver for IoctlTokenResolver
{
    fn resolve(&self, challenge: &[u8]) -> Result<Vec<u8>, RaTlsError> {
        if challenge.len() != rust_rsi::CHALLENGE_LEN as usize {
            return Err(RaTlsError::GenericTokenResolverError("Challange needs to be exactly 64 bytes".into()));
        }

        match rust_rsi::attestation_token(&challenge.try_into().unwrap())
        {
            Err(e) => Err(RaTlsError::GenericTokenResolverError(Box::new(e))),
            Ok(v) => Ok(v),
        }
    }
}
