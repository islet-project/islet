#[derive(Debug)]
pub struct SWComponent {
    pub name: (u16, String),
    pub measurement: (u16, [u8; 32]),
    pub version: (u16, String),
    pub signer_id: (u16, [u8; 32]),
    pub hash_algo: (u16, String),
}
