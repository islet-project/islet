pub const SMC_SUCCESS: usize = 0;

pub type SecureMonitorCall = &'static dyn Caller;

pub enum Code {
    MarkRealm,
    MarkNonSecure,
}

pub trait Caller {
    fn convert(&self, command: Code) -> usize;
    fn call(&self, command: usize, args: &[usize]) -> [usize; 8];
}
