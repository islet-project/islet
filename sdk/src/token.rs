use crate::claim::{
    platform::{SWComponent0, SWComponent1},
    Claim,
};

#[derive(Debug)]
pub struct Platform {
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
