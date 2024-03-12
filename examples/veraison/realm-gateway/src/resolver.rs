use ratls::{InternalTokenResolver, RaTlsError};
use rust_rsi;

pub struct IoctlTokenResolver
{
    pub user_data: [u8; 64],
}

impl IoctlTokenResolver {
    pub const fn new() -> Self {
        Self {
            user_data: [0; 64],
        }
    }
}

impl InternalTokenResolver for IoctlTokenResolver
{
    fn resolve(&self, challenge: &[u8]) -> Result<Vec<u8>, RaTlsError> {
        if challenge.len() != rust_rsi::CHALLENGE_LEN as usize {
            return Err(RaTlsError::GenericTokenResolverError("Challange needs to be exactly 64 bytes".into()));
        }

        // REM[0] setting (index-1)
        match rust_rsi::measurement_extend(1, &self.user_data)
        {
            Err(e) => {
                return Err(RaTlsError::GenericTokenResolverError(Box::new(e)));
            },
            Ok(_) => {},
        }

        match rust_rsi::attestation_token(&challenge.try_into().unwrap())
        {
            Err(e) => Err(RaTlsError::GenericTokenResolverError(Box::new(e))),
            Ok(v) => Ok(v),
        }
    }
}
