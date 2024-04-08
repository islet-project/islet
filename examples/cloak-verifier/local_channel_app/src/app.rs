use prusti_contracts::*;
use common::ioctl::{cloak_connect, cloak_read, cloak_write, cloak_set_status};
use std::{thread, time};

pub struct LocalChannelApp<S, M> where
    S: LocalChannelState,
    M: SharedMemoryState
{
    id: usize,
    shared_memory: SharedMemory<M>,
    state: S,  // (0) Created --> (1) Connected --> (2) Received --> (3) Established
}

#[derive(Clone, Copy)]
struct SharedMemory<M> where
    M: SharedMemoryState
{
    ipa: usize,
    state: M,  // (0) Unmapped --> (1) ReadOnly --> (2) ReadWrite
}
impl SharedMemory<Unmapped> {
    #[ensures((&result).state.is_unmapped())]
    const fn new(ipa: usize) -> SharedMemory<Unmapped> {
        Self {
            ipa: ipa,
            state: Unmapped,
        }
    }
}
impl SharedMemory<Unmapped> {
    #[requires(self.state.is_unmapped())]
    #[ensures((&result).state.is_read_only() && old(self).ipa == (&result).ipa)]
    fn into_read_only(self) -> SharedMemory<ReadOnly> {
        SharedMemory {
            ipa: self.ipa,
            state: ReadOnly,
        }
    }
}
impl SharedMemory<ReadOnly> {
    #[requires(self.state.is_read_only())]
    #[ensures((&result).state.is_read_write() && old(self).ipa == (&result).ipa)]
    fn into_read_write(self) -> SharedMemory<ReadWrite> {
        SharedMemory {
            ipa: self.ipa,
            state: ReadWrite,
        }
    }

    #[requires(self.state.is_read_only())]
    #[ensures(self.state.is_read_only())]
    fn read_only(&self, id: usize, data: &mut [u8; 4096]) -> bool {
        match cloak_read(id, data) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
impl SharedMemory<ReadWrite> {
    #[requires(self.state.is_read_write())]
    #[ensures(self.state.is_read_write())]
    pub fn read(&self, id: usize, data: &mut [u8; 4096]) -> bool {
        match cloak_read(id, data) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[requires(self.state.is_read_write())]
    #[ensures(self.state.is_read_write())]
    pub fn write(&self, id: usize, data: &[u8; 4096]) -> bool {
        match cloak_write(id, data) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl<S: LocalChannelState, M: SharedMemoryState> LocalChannelApp<S, M> {
    #[ensures( ((&result).state.is_created()) && ((&result).shared_memory.state.is_unmapped()) )]
    pub const fn new(id: usize) -> LocalChannelApp<Created, Unmapped> {
        LocalChannelApp {
            id: id,
            shared_memory: SharedMemory::<Unmapped>::new(0),
            state: Created,
        }
    }
}

#[pure]
fn ensures_connect(res: &Option<LocalChannelApp<Connected, ReadOnly>>) -> bool {
    match res {
        Some(app) => {
            app.state.is_connected() && app.shared_memory.state.is_read_only()
        },
        None => true,
    }
}

impl LocalChannelApp<Created, Unmapped> {
    #[requires( ((&self).state.is_created()) && ((&self).shared_memory.state.is_unmapped()) )]
    #[ensures(ensures_connect(&result))]
    //#[ensures( ((&result).state.is_connected()) && ((&result).shared_memory.state.is_read_only()) )]
    pub fn connect(self) -> Option<LocalChannelApp<Connected, ReadOnly>> {
        match cloak_connect(self.id) {
            Ok(_) => {
                match cloak_set_status(self.id, 1) {
                    Ok(_) => println!("cloak_set_status success"),
                    Err(_) => println!("cloak_set_status fail"),
                }

                Some(LocalChannelApp {
                    id: self.id,
                    shared_memory: self.shared_memory.into_read_only(),
                    state: Connected,
                })
            },
            Err(_) => None,
        }    
    }
}

impl LocalChannelApp<Connected, ReadOnly> {
    #[requires( ((&self).state.is_connected()) && ((&self).shared_memory.state.is_read_only()) )]
    #[ensures( ((&result).state.is_received()) && ((&result).shared_memory.state.is_read_only()) )]
    pub fn wait_for_signed_cert(self) -> LocalChannelApp<Received, ReadOnly> {
        loop {
            let mut data: [u8; 4096] = [0; 4096];
            let read_res = self.shared_memory.read_only(self.id, &mut data);
            if read_res == false {
                thread::sleep(time::Duration::from_millis(1000)); // 1s
                continue;
            }
            if data[0] == 0x01 {
                break;
            }
            thread::sleep(time::Duration::from_millis(1000));
        }
        
        LocalChannelApp {
            id: self.id,
            shared_memory: self.shared_memory,
            state: Received,
        }
    }
}

#[pure]
fn ensures_establish(res: &Option<LocalChannelApp<Established, ReadWrite>>) -> bool {
    match res {
        Some(app) => {
            app.state.is_established() && app.shared_memory.state.is_read_write()
        },
        None => true,
    }
}

impl LocalChannelApp<Received, ReadOnly> {
    #[requires( ((&self).state.is_received()) && ((&self).shared_memory.state.is_read_only()) )]
    #[ensures(ensures_establish(&result))]
    //#[ensures( ((&result).state.is_established()) && ((&result).shared_memory.state.is_read_write()) )]
    pub fn establish(self) -> Option<LocalChannelApp<Established, ReadWrite>> {
        let mut data: [u8; 4096] = [0; 4096];
        let read_res = self.shared_memory.read_only(self.id, &mut data);
        if read_res == false {
            return None;
        }
        if data[0] != 0x01 { // TODO: check a signed cert!
            return None;
        }
        
        Some(LocalChannelApp {
            id: self.id,
            shared_memory: self.shared_memory.into_read_write(),
            state: Established,
        })
    }
}

impl LocalChannelApp<Established, ReadWrite> {
    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) )]
    pub fn perform(&self) {
        //println!("perform! {}", self.id);
    }

    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) )]
    //#[ensures( ((&result).state.is_established()) && ((&result).shared_memory.state.is_read_write()) )]
    pub fn read(&self, data: &mut [u8; 4096]) -> bool {
        self.shared_memory.read(self.id, data)
    }

    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) )]
    //#[ensures( ((&result).state.is_established()) && ((&result).shared_memory.state.is_read_write()) )]
    pub fn write(&self, data: &[u8; 4096]) -> bool {
        self.shared_memory.write(self.id, data)
    }
}

// Typestate definitions for LocalChannelState
pub struct Start;
pub struct Created;
pub struct Connected;
pub struct Received;
pub struct Established;

pub trait LocalChannelState {
    #[pure]
    fn is_start(&self) -> bool { false }

    #[pure]
    fn is_created(&self) -> bool { false }

    #[pure]
    fn is_connected(&self) -> bool { false }

    #[pure]
    fn is_received(&self) -> bool { false }

    #[pure]
    fn is_established(&self) -> bool { false }
}

#[refine_trait_spec]
impl LocalChannelState for Start {
    #[pure]
    #[ensures(result == true)]
    fn is_start(&self) -> bool { true }
}

#[refine_trait_spec]
impl LocalChannelState for Created {
    #[pure]
    #[ensures(result == true)]
    fn is_created(&self) -> bool { true }
}

#[refine_trait_spec]
impl LocalChannelState for Connected {
    #[pure]
    #[ensures(result == true)]
    fn is_connected(&self) -> bool { true }
}

#[refine_trait_spec]
impl LocalChannelState for Received {
    #[pure]
    #[ensures(result == true)]
    fn is_received(&self) -> bool { true }
}

#[refine_trait_spec]
impl LocalChannelState for Established {
    #[pure]
    #[ensures(result == true)]
    fn is_established(&self) -> bool { true }
}

// Typestate definitions for SharedMemoryState
pub struct Unmapped;
pub struct ReadOnly;
pub struct ReadWrite;

pub trait SharedMemoryState {
    #[pure]
    fn is_unmapped(&self) -> bool { false }

    #[pure]
    fn is_read_only(&self) -> bool { false }

    #[pure]
    fn is_read_write(&self) -> bool { false }
}

#[refine_trait_spec]
impl SharedMemoryState for Unmapped {
    #[pure]
    #[ensures(result == true)]
    fn is_unmapped(&self) -> bool { true }
}

#[refine_trait_spec]
impl SharedMemoryState for ReadOnly {
    #[pure]
    #[ensures(result == true)]
    fn is_read_only(&self) -> bool { true }
}

#[refine_trait_spec]
impl SharedMemoryState for ReadWrite {
    #[pure]
    #[ensures(result == true)]
    fn is_read_write(&self) -> bool { true }
}
