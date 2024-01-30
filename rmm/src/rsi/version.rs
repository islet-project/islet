use crate::rsi;

pub fn decode_version(version: usize) -> (usize, usize) {
    let major = (version & 0x7fff0000) >> 16;
    let minor = version & 0xffff;

    (major, minor)
}

pub fn encode_version() -> usize {
    (rsi::ABI_VERSION_MAJOR << 16) | rsi::ABI_VERSION_MINOR
}
