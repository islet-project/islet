#[derive(Debug)]
pub struct Report {
    pub platform: token::Platform,
    pub realm: token::Realm,
}

pub mod token {
    use crate::claim::{
        platform::{SWComponent0, SWComponent1},
        Claim,
    };

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

    #[derive(Debug)]
    pub struct Realm {
        pub claims_len: u64,
        pub challenge: Claim<[u8; 64]>,
        pub rpv: Claim<[u8; 64]>,
        pub public_key: Claim<[u8; 97]>,
        pub hash_algo: Claim<String>,
        pub public_key_hash_algo: Claim<String>,
        pub rim: Claim<[u8; 32]>,
        pub rem: Claim<[u8; 32]>,
    }
}
