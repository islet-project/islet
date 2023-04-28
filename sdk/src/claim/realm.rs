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
