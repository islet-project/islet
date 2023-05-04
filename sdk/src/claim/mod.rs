pub mod platform;
pub mod realm;

pub use self::platform::Platform as PlatformToken;
pub use self::realm::Realm as RealmToken;

pub type RealmSignature = Claim;
pub type PlatformSignature = Claim;

#[derive(Debug)]
pub enum Value {
    U16(u16),
    String(String),
    Bytes(Vec<u8>),
}

#[derive(Debug)]
pub struct Claim {
    pub label: u16,
    pub title: &'static str,
    pub value: Value,
}

#[derive(Debug)]
pub struct Claims {
    pub realm_sig: RealmSignature,
    pub realm_tok: RealmToken,
    pub plat_sig: PlatformSignature,
    pub plat_tok: PlatformToken,
}
