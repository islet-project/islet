pub type SecureMonitorCall = &'static dyn Caller;
static mut SMC: Option<SecureMonitorCall> = None;

pub enum Code {
    MarkRealm,
    MarkNonSecure,
}

#[allow(unused_must_use)]
pub fn set_instance(smc: SecureMonitorCall) {
    unsafe {
        if SMC.is_none() {
            SMC = Some(smc);
        }
    };
}

pub fn instance() -> Option<SecureMonitorCall> {
    unsafe { SMC }
}

pub trait Caller {
    fn convert(&self, command: Code) -> usize;
    fn call(&self, command: usize, args: [usize; 4]) -> [usize; 8];
}
