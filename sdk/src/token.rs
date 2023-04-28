#[derive(Debug)]
pub struct Platform {
    pub profile: (u16, String),
    pub challenge: (u16, [u8; 32]),
    pub implementation_id: (u16, [u8; 64]),
    pub instance_id: (u16, [u8; 33]),
    pub config: (u16, [u8; 33]),
    pub lifecycle: (u16, u16),
    pub hash_algo: (u16, String),
    pub sw_components: (u16, SWComponent0, SWComponent1, SWComponent1, SWComponent1),
    pub verification_service: (u16, String),
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
