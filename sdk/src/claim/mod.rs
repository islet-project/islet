pub mod platform;
pub mod realm;

#[derive(Debug)]
pub struct Claim<T: std::fmt::Debug> {
    pub label: u16,
    pub value: T,
}
