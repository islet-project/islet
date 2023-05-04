use super::Claim;

#[derive(Debug)]
pub struct Realm {
    pub claims_len: u64,
    pub challenge: Claim,
    pub rpv: Claim,
    pub public_key: Claim,
    pub hash_algo: Claim,
    pub public_key_hash_algo: Claim,
    pub rim: Claim,
    pub rem: Claim,
}
