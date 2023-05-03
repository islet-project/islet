pub mod platform;
pub mod realm;

pub use self::platform::Platform as PlatformToken;
pub use self::realm::Realm as RealmToken;

pub type RealmSignature = Claim<[u8; 96]>;
pub type PlatformSignature = Claim<[u8; 64]>;

#[derive(Debug)]
pub struct Claim<T: std::fmt::Debug> {
    pub label: u16,
    pub value: T,
}

#[derive(Debug)]
pub struct Claims {
    pub realm_sig: RealmSignature,
    pub realm_tok: RealmToken,
    pub plat_sig: PlatformSignature,
    pub plat_tok: PlatformToken,
}
