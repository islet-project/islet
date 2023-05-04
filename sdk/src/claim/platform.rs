use super::Claim;

#[derive(Debug)]
pub struct Platform {
    pub claims_len: u64,
    pub profile: Claim,
    pub challenge: Claim,
    pub implementation_id: Claim,
    pub instance_id: Claim,
    pub config: Claim,
    pub lifecycle: Claim,
    pub hash_algo_id: Claim,
    pub sw_components: (SWComponent0, SWComponent1, SWComponent1, SWComponent1),
    pub verification_service: Claim,
}

#[derive(Debug)]
pub struct SWComponent0 {
    pub name: (u16, String),
    pub measurement: (u16, [u8; 32]),
    pub version: (u16, String),
    pub signer_id: (u16, [u8; 32]),
    pub hash_algo: (u16, String),
}

#[derive(Debug)]
pub struct SWComponent1 {
    pub name: (u16, String),
    pub measurement: (u16, [u8; 32]),
    pub version: (u16, String),
    pub signer_id: (u16, [u8; 32]),
}
