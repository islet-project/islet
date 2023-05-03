use super::Claim;
use crate::config;

#[derive(Debug)]
pub struct Realm {
    pub claims_len: u64,
    pub challenge: Claim<[u8; config::CHALLENGE_SIZE]>,
    pub rpv: Claim<[u8; 64]>,
    pub public_key: Claim<[u8; 97]>,
    pub hash_algo: Claim<String>,
    pub public_key_hash_algo: Claim<String>,
    pub rim: Claim<[u8; 32]>,
    pub rem: Claim<[u8; 32]>,
}

#[repr(u16)]
pub enum Label {
    Challenge = 10,
    RPV = 44235,
    PublicKey = 44237,
    HashAlgo = 44240,
    PublicKeyHashAlgo = 44236,
    RIM = 44238,
    REM = 44239,
}
