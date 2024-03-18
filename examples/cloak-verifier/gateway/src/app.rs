use prusti_contracts::*;
use common::ioctl::{cloak_create, cloak_gen_report, cloak_write, cloak_read};
use sha2::{Sha512, Digest};
use crate::ttp;
use std::io::Write;
use std::sync::Arc;

pub struct Gateway<S, M> where
    S: LocalChannelState,
    M: SharedMemoryState,
    //T: ChannelTTPState
{
    id: usize,
    shared_memory: SharedMemory<M>,
    //channel_ttp: ChannelTTP<T>,  // (0) Initialized --> (1) Established --> (2) Verified --> (3) Destroyed
    state: S,  // (0) Initialized --> (1) Created --> (2) Connected --> (3) Established
}

/*
struct ChannelTTP<T> wheref
    T: ChannelTTPState
{
    state: T,
}
impl ChannelTTP<Initialized> {
    #[ensures((&result).state.is_initialized())]
    pub const fn new() -> ChannelTTP<Initialized> {
        Self {
            state: Initialized,
        }
    }
}
impl ChannelTTP<Initialized> {
    #[requires(self.state.is_initialized())]
    #[ensures((&result).state.is_established())]
    pub fn establish(self) -> ChannelTTP<Established> {
        ChannelTTP {
            state: Established,
        }
    }
}
impl ChannelTTP<Established> {
    #[requires(self.state.is_established())]
    #[ensures((&result).state.is_verified())]
    pub fn read_signed_cert(self) -> ChannelTTP<Verified> {
        ChannelTTP {
            state: Verified,
        }
    }
}
impl ChannelTTP<Verified> {
    #[requires(self.state.is_verified())]
    #[ensures((&result).state.is_destroyed())]
    pub fn verify(self) -> ChannelTTP<Destroyed> {
        ChannelTTP {
            state: Destroyed,
        }
    }
} */

#[derive(Clone, Copy)]
struct SharedMemory<M> where
    M: SharedMemoryState
{
    ipa: usize,
    state: M,  // (0) Unmapped --> (1) WriteOnly --> (2) ReadWrite
}
impl SharedMemory<Unmapped> {
    #[ensures((&result).state.is_unmapped())]
    pub const fn new(ipa: usize) -> SharedMemory<Unmapped> {
        Self {
            ipa: ipa,
            state: Unmapped,
        }
    }
}
impl SharedMemory<Unmapped> {
    #[requires(self.state.is_unmapped())]
    #[ensures((&result).state.is_write_only() && old(self).ipa == (&result).ipa)]
    pub fn into_write_only(self) -> SharedMemory<WriteOnly> {
        SharedMemory {
            ipa: self.ipa,
            state: WriteOnly,
        }
    }
}
impl SharedMemory<WriteOnly> {
    #[requires(self.state.is_write_only())]
    #[ensures((&result).state.is_read_write() && old(self).ipa == (&result).ipa)]
    pub fn into_read_write(self) -> SharedMemory<ReadWrite> {
        SharedMemory {
            ipa: self.ipa,
            state: ReadWrite,
        }
    }

    #[requires(self.state.is_write_only())]
    #[ensures(self.state.is_write_only())]
    pub fn write_only(&self, id: usize, data: &[u8; 4096]) -> bool {
        match cloak_write(id, data) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
impl SharedMemory<ReadWrite> {
    #[requires(self.state.is_read_write())]
    #[ensures(self.state.is_read_write())]
    pub fn write(&self, id: usize, data: &[u8; 4096]) -> bool {
        match cloak_write(id, data) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[requires(self.state.is_read_write())]
    #[ensures(self.state.is_read_write())]
    pub fn read(&self, id: usize, data: &mut [u8; 4096]) -> bool {
        // MIRAI: data should be tagged as "secret" for taint-analysis
        //        "secret" can be untagged by encryption.
        match cloak_read(id, data) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

impl<S: LocalChannelState, M: SharedMemoryState> Gateway<S, M> {
    #[ensures( ((&result).state.is_initialized()) && ((&result).shared_memory.state.is_unmapped()) )]
    pub const fn new() -> Gateway<Initialized, Unmapped> {
        Gateway {
            id: 0,
            shared_memory: SharedMemory::<Unmapped>::new(0),
            state: Initialized,
        }
    }
}

#[pure]
fn ensures_create(res: &Option<Gateway<Created, Unmapped>>) -> bool {
    match res {
        Some(gw) => {
            gw.state.is_created() && gw.shared_memory.state.is_unmapped()
        },
        None => true,
    }
}

impl Gateway<Initialized, Unmapped> {
    #[requires( ((&self).state.is_initialized()) && ((&self).shared_memory.state.is_unmapped()) )]
    #[ensures(ensures_create(&result))]
    //#[ensures( ((&result).state.is_created()) && ((&result).shared_memory.state.is_unmapped()) )]
    pub fn create(self) -> Option<Gateway<Created, Unmapped>> {
        // do something here! create()

        match cloak_create(self.id) {
            Ok(_) => {
                Some(Gateway {
                    id: self.id,
                    shared_memory: self.shared_memory,
                    state: Created,
                })
            },
            Err(_) => {
                None
            },
        }
    }
}

impl Gateway<Created, Unmapped> {
    #[requires( ((&self).state.is_created()) && ((&self).shared_memory.state.is_unmapped()) )]
    #[ensures( ((&result).state.is_connected()) && ((&result).shared_memory.state.is_write_only()) )]
    pub fn wait_for_app(self) -> Gateway<Connected, WriteOnly> {
        // do something here!
        // -- (1) polling until status() == connected
        
        Gateway {
            id: self.id,
            shared_memory: self.shared_memory.into_write_only(),
            state: Connected,
        }
    }
}

#[pure]
fn ensures_establish(res: &Option<Gateway<Established, ReadWrite>>) -> bool {
    match res {
        Some(gw) => {
            gw.state.is_established() && gw.shared_memory.state.is_read_write()
        },
        None => true,
    }
}

impl Gateway<Connected, WriteOnly> {
    #[requires( ((&self).state.is_connected()) && ((&self).shared_memory.state.is_write_only()) )]
    #[ensures(ensures_establish(&result))]
    //#[ensures( ((&result).state.is_established()) && ((&result).shared_memory.state.is_read_write()) )]
    pub fn establish(self) -> Option<Gateway<Established, ReadWrite>> {
        // do something here!
        // -- (1) communicate with TTP
        // -- (2) receive a signed cert
        // -- (3) send the cert to App
        // -- (4) validate the cert and change it to Established.
        /*
        let channel_ttp = self.channel_ttp;
        let channel_ttp = channel_ttp.establish();
        let channel_ttp = channel_ttp.read_signed_cert();
        let channel_ttp = channel_ttp.verify(); */

        // 1. generate a report (App and GW)
        let mut counterpart_token: Vec<u8> = Vec::new();
        match cloak_gen_report(self.id) {
            Ok(token) => {
                counterpart_token.clone_from(&token);
            },
            Err(_) => {
                return None;
            },
        }
        let mut hasher = Sha512::new();
        hasher.update(&counterpart_token);
        let _counterpart_hash = hasher.finalize()[..].to_vec();

        // 2. interact with TTP
        // Gateway(RIM) + App(REM)
        // TODO: debugging network connection-:
        /*
        let root_ca = "root-ca.crt";
        let server_url = "193.168.10.15:1337";
        let server_name = "localhost";

        let client = ttp::RaTlsClient::new(ttp::ClientMode::AttestedClient {
            rem: counterpart_hash,
            root_ca_path: root_ca.to_string()
        });
        if client.is_err() {
            println!("ttp::RaTlsClient::new error");
            return None;
        }
        println!("ttp::RaTlsClient::new success");

        let client = client.unwrap();
        let mut connection = client.connect(server_url.to_string(), server_name.to_string());
        if connection.is_err() {
            println!("client.connect error");
            return None;
        }
        println!("client.connect success");

        let mut connection = connection.unwrap();
        let write_res = write!(connection.stream(), "GIT");
        if write_res.is_err() {
            println!("connection.write error");
            return None;
        }
        println!("connection.write success"); */

        // 3. write result to "shared memory"
        let data: [u8; 4096] = [1; 4096];
        let write_res = self.shared_memory.write_only(self.id, &data);
        if write_res == false {
            return None;
        }
        
        Some(Gateway {
            id: self.id,
            shared_memory: self.shared_memory.into_read_write(),
            state: Established,
        })
    }
}

impl Gateway<Established, ReadWrite> {
    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) )]
    pub fn perform(&self) {
        //println!("perform! {}", self.id);
    }
}

// Typestate definitions for ChannelTTPState
/*
pub struct Verified;
pub struct Destroyed;

pub trait ChannelTTPState {
    #[pure]
    fn is_initialized(&self) -> bool { false }

    #[pure]
    fn is_established(&self) -> bool { false }

    #[pure]
    fn is_verified(&self) -> bool { false }

    #[pure]
    fn is_destroyed(&self) -> bool { false }
} 

#[refine_trait_spec]
impl ChannelTTPState for Initialized {
    #[pure]
    #[ensures(result == true)]
    fn is_initialized(&self) -> bool { true }
}

#[refine_trait_spec]
impl ChannelTTPState for Established {
    #[pure]
    #[ensures(result == true)]
    fn is_established(&self) -> bool { true }
}

#[refine_trait_spec]
impl ChannelTTPState for Verified {
    #[pure]
    #[ensures(result == true)]
    fn is_verified(&self) -> bool { true }
}

#[refine_trait_spec]
impl ChannelTTPState for Destroyed {
    #[pure]
    #[ensures(result == true)]
    fn is_destroyed(&self) -> bool { true }
} */

// Typestate definitions for LocalChannelState
pub struct Initialized;
pub struct Created;
pub struct Connected;
pub struct Established;

pub trait LocalChannelState {
    #[pure]
    fn is_initialized(&self) -> bool { false }

    #[pure]
    fn is_created(&self) -> bool { false }

    #[pure]
    fn is_connected(&self) -> bool { false }

    #[pure]
    fn is_established(&self) -> bool { false }
}

#[refine_trait_spec]
impl LocalChannelState for Initialized {
    #[pure]
    #[ensures(result == true)]
    fn is_initialized(&self) -> bool { true }
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
impl LocalChannelState for Established {
    #[pure]
    #[ensures(result == true)]
    fn is_established(&self) -> bool { true }
}

// Typestate definitions for SharedMemoryState
pub struct Unmapped;
pub struct WriteOnly;
pub struct ReadWrite;

pub trait SharedMemoryState {
    #[pure]
    fn is_unmapped(&self) -> bool { false }

    #[pure]
    fn is_write_only(&self) -> bool { false }

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
impl SharedMemoryState for WriteOnly {
    #[pure]
    #[ensures(result == true)]
    fn is_write_only(&self) -> bool { true }
}

#[refine_trait_spec]
impl SharedMemoryState for ReadWrite {
    #[pure]
    #[ensures(result == true)]
    fn is_read_write(&self) -> bool { true }
}
