use super::Claim;

#[derive(Debug)]
pub struct Platform {
    pub claims_len: u64,
    pub profile: Claim<String>,
    pub challenge: Claim<[u8; 32]>,
    pub implementation_id: Claim<[u8; 64]>,
    pub instance_id: Claim<[u8; 33]>,
    pub config: Claim<[u8; 33]>,
    pub lifecycle: Claim<u16>,
    pub hash_algo: Claim<String>,
    pub sw_components: Claim<(SWComponent0, SWComponent1, SWComponent1, SWComponent1)>,
    pub verification_service: Claim<String>,
}

#[repr(u16)]
pub enum Label {
    Profile = 265,
    Challenge = 10,
    ImplementationId = 2396,
    InstanceId = 256,
    Config = 2401,
    Lifecycle = 2395,
    HashAlgo = 2402,
    SWComponents = 2399,
    VerificationService = 2400,
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
