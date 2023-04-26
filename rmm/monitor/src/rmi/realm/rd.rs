use core::mem::ManuallyDrop;

pub struct Rd {
    realm_id: usize,
    state: State,
}

impl Rd {
    pub unsafe fn new(rd_addr: usize, realm_id: usize) -> ManuallyDrop<&'static mut Rd> {
        let rd: &mut Rd = &mut *(rd_addr as *mut Rd);
        *rd = Default::default();
        rd.realm_id = realm_id;
        ManuallyDrop::new(rd)
    }

    pub unsafe fn into(rd_addr: usize) -> ManuallyDrop<&'static mut Rd> {
        let rd: &mut Rd = &mut *(rd_addr as *mut Rd);
        ManuallyDrop::new(rd)
    }

    pub fn id(&self) -> usize {
        self.realm_id
    }

    pub fn at_state(&self, compared: State) -> bool {
        self.state == compared
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
