use core::mem::ManuallyDrop;

pub struct Rd {
    pub realm_id: usize,
    pub state: State,
}

impl Rd {
    pub unsafe fn new(rd_addr: usize) -> ManuallyDrop<&'static mut Rd> {
        let rd: &mut Rd = &mut *(rd_addr as *mut Rd);
        *rd = Default::default();
        ManuallyDrop::new(rd)
    }

    pub unsafe fn into(rd_addr: usize) -> ManuallyDrop<&'static mut Rd> {
        let rd: &mut Rd = &mut *(rd_addr as *mut Rd);
        ManuallyDrop::new(rd)
    }
}

impl Default for Rd {
    fn default() -> Self {
        Self {
            realm_id: 0,
            state: State::New,
        }
    }
}

impl Drop for Rd {
    fn drop(&mut self) {}
}

#[derive(Debug, PartialEq)]
pub enum State {
    Null,
    New,
    Active,
    SystemOff,
}
