/*
use ratls::{InternalTokenResolver, RaTlsError};
//use common::{measurement_extend, attestation_token};

const CHALLENGE_LEN: u16 = 0x40;

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
        if challenge.len() != CHALLENGE_LEN as usize {
            return Err(RaTlsError::GenericTokenResolverError("Challange needs to be exactly 64 bytes".into()));
        }
        Ok(Vec::new())

        // REM[0] setting (index-1)
        match measurement_extend(1, &self.user_data)
        {
            Err(_) => {
                return Err(RaTlsError::GenericTokenResolverError("extend".into()));
            },
            Ok(_) => {},
        }

        match attestation_token(&challenge.try_into().unwrap())
        {
            Err() => Err(RaTlsError::GenericTokenResolverError("token".into())),
            Ok(v) => Ok(v),
        }
    }
}
*/