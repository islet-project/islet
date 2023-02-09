pub type RMMConfig = &'static dyn Config;
static mut RMMCONFIG: Option<RMMConfig> = None;

#[allow(unused_must_use)]
pub fn set_instance(config: RMMConfig) {
    unsafe {
        if RMMCONFIG.is_none() {
            RMMCONFIG = Some(config);
        }
    };
}

pub fn instance() -> Option<RMMConfig> {
    unsafe { RMMCONFIG }
}

pub trait Config {
    fn abi_version(&self) -> usize;
}
