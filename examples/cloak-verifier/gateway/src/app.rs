use prusti_contracts::*;
use common::ioctl::{cloak_create, cloak_gen_report, cloak_write, cloak_read};
use sha2::{Sha512, Digest};

use std::sync::Arc;
use crate::ttp;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

// MIRAI
#[cfg_attr(mirai, allow(incomplete_features), feature(generic_const_exprs))]

#[cfg(mirai)]
use mirai_annotations::{TagPropagation, TagPropagationSet};

#[cfg(mirai)]
struct TaintedKind<const MASK: TagPropagationSet> {}

#[cfg(mirai)]
const TAINTED_MASK: TagPropagationSet = tag_propagation_set!(TagPropagation::SubComponent);

#[cfg(mirai)]
type Tainted = TaintedKind<TAINTED_MASK>;  // Attach "Tainted" for secret
#[cfg(not(mirai))]
type Tainted = ();

#[cfg(mirai)]
struct SanitizedKind<const MASK: TagPropagationSet> {}

#[cfg(mirai)]
const SANITIZED_MASK: TagPropagationSet = tag_propagation_set!(TagPropagation::SubComponent);

#[cfg(mirai)]
type Sanitized = SanitizedKind<SANITIZED_MASK>;
#[cfg(not(mirai))]
type Sanitized = ();  // Attach "Sanitized" when secret is encrypted


pub struct Gateway<S, M, R> where
    S: LocalChannelState,
    M: SharedMemoryState,
    R: RemoteChannelState,
    //RR: RemoteChannelRxState,
    //T: ChannelTTPState
{
    id: usize,
    shared_memory: SharedMemory<M>,
    //channel_ttp: ChannelTTP<T>,  // (0) Initialized --> (1) Established --> (2) Verified --> (3) Destroyed
    state: S,  // (0) Initialized --> (1) Created --> (2) Connected --> (3) Established
    rc_state: R,
}

/*
struct ChannelTTP<T> where
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

pub struct RemoteChannel
{
    stream: TcpStream,
}
impl RemoteChannel {
    fn connect() -> Option<RemoteChannel> {
        if let Ok(stream) = TcpStream::connect("193.168.10.15:1999") {
            Some(RemoteChannel {
                stream: stream
            })
        }
        else {
            None
        }
    }
}


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

#[pure]
fn ensures_shared_memory_read(res: &Option<Data<UnencryptedLocalData>>) -> bool {
    match res {
        Some(d) => d.state.is_unencrypted_local_data(),
        None => true,
    }
}

impl SharedMemory<ReadWrite> {
    #[requires(self.state.is_read_write() && data.state.is_unencrypted_remote_data())]
    #[ensures(self.state.is_read_write())]
    fn write(&self, id: usize, data: &Data<UnencryptedRemoteData>) -> bool {
        match cloak_write(id, &data.data) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[requires(self.state.is_read_write())]
    #[ensures(ensures_shared_memory_read(&result))]
    fn read(&self, id: usize) -> Option<Data<UnencryptedLocalData>> {
        // MIRAI: data should be tagged as "secret" for taint-analysis
        //        "secret" can be untagged by encryption.
        let mut data: [u8; 4096] = [0; 4096];

        match cloak_read(id, &mut data) {
            Ok(_) => {
                Some(Data::<Uninitialized>::new_from_local(data))
            },
            Err(_) => None,
        }
    }
}

pub struct Data<S> where
    S: DataState
{
    data: [u8; 4096],
    state: S,
}

impl Data<Uninitialized> {
    #[ensures((&result).state.is_unencrypted_local_data())]
    const fn new_from_local(data: [u8; 4096]) -> Data<UnencryptedLocalData> {
        Data {
            data: data,
            state: UnencryptedLocalData,
        }
    }

    #[ensures((&result).state.is_encrypted_remote_data())]
    const fn new_from_remote(data: [u8; 4096]) -> Data<EncryptedRemoteData> {
        Data {
            data: data,
            state: EncryptedRemoteData,
        }
    }
}

impl Data<UnencryptedLocalData> {
    #[requires(self.state.is_unencrypted_local_data())]
    #[ensures((&result).state.is_encrypted_local_data())]
    fn encrypt(self) -> Data<EncryptedLocalData> {
        Data {
            data: self.data,
            state: EncryptedLocalData,
        }
    }
}

impl Data<EncryptedRemoteData> {
    #[requires(self.state.is_encrypted_remote_data())]
    #[ensures((&result).state.is_unencrypted_remote_data())]
    fn decrypt(self) -> Data<UnencryptedRemoteData> {
        Data {
            data: self.data,
            state: UnencryptedRemoteData,
        }
    }
}

impl<S: LocalChannelState, M: SharedMemoryState, R: RemoteChannelState> Gateway<S, M, R> {
    #[ensures( ((&result).state.is_initialized()) && ((&result).shared_memory.state.is_unmapped()) && ((&result).rc_state.is_rc_initialized()) )]
    pub const fn new() -> Gateway<Initialized, Unmapped, Initialized> {
        Gateway {
            id: 0,
            shared_memory: SharedMemory::<Unmapped>::new(0),
            state: Initialized,
            rc_state: Initialized,
        }
    }
}

#[pure]
fn ensures_create(res: &Option<Gateway<Created, Unmapped, Initialized>>) -> bool {
    match res {
        Some(gw) => {
            gw.state.is_created() && gw.shared_memory.state.is_unmapped() && gw.rc_state.is_rc_initialized()
        },
        None => true,
    }
}

impl Gateway<Initialized, Unmapped, Initialized> {
    #[requires( ((&self).state.is_initialized()) && ((&self).shared_memory.state.is_unmapped()) && ((&self).rc_state.is_rc_initialized()) )]
    #[ensures(ensures_create(&result))]
    //#[ensures( ((&result).state.is_created()) && ((&result).shared_memory.state.is_unmapped()) )]
    pub fn create(self) -> Option<Gateway<Created, Unmapped, Initialized>> {
        // do something here! create()

        match cloak_create(self.id) {
            Ok(_) => {
                Some(Gateway {
                    id: self.id,
                    shared_memory: self.shared_memory,
                    state: Created,
                    rc_state: Initialized,
                })
            },
            Err(_) => {
                None
            },
        }
    }
}

impl Gateway<Created, Unmapped, Initialized> {
    #[requires( ((&self).state.is_created()) && ((&self).shared_memory.state.is_unmapped()) && ((&self).rc_state.is_rc_initialized()) )]
    #[ensures( ((&result).state.is_connected()) && ((&result).shared_memory.state.is_write_only()) && ((&result).rc_state.is_rc_initialized()))]
    pub fn wait_for_app(self) -> Gateway<Connected, WriteOnly, Initialized> {
        // do something here!
        // -- (1) polling until status() == connected
        
        Gateway {
            id: self.id,
            shared_memory: self.shared_memory.into_write_only(),
            state: Connected,
            rc_state: Initialized,
        }
    }
}

#[pure]
fn ensures_establish(res: &Option< (Gateway<Established, ReadWrite, Established>, RemoteChannel) >) -> bool {
    match res {
        Some(gw) => {
            gw.0.state.is_established() && gw.0.shared_memory.state.is_read_write() && gw.0.rc_state.is_rc_established()
        },
        None => true,
    }
}

impl Gateway<Connected, WriteOnly, Initialized> {
    #[requires( ((&self).state.is_connected()) && ((&self).shared_memory.state.is_write_only()) && ((&self).rc_state.is_rc_initialized()) )]
    #[ensures(ensures_establish(&result))]
    //#[ensures( ((&result).state.is_established()) && ((&result).shared_memory.state.is_read_write()) )]
    pub fn establish(self) -> Option< (Gateway<Established, ReadWrite, Established>, RemoteChannel) > {
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
        let counterpart_hash = hasher.finalize()[..].to_vec();

        // 2. interact with TTP
        // Gateway(RIM) + App(REM)
        let root_ca = "root.crt";
        let server_url = "193.168.10.15:1337";
        let server_name = "localhost";

        // 2-1. connect to TTP via RA-TLS
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
        let connection = client.connect(server_url.to_string(), server_name.to_string());
        if connection.is_err() {
            println!("client.connect error");
            return None;
        }
        println!("client.connect success");

        // 2-2. send an attestation report
        let mut connection = connection.unwrap();
        let write_data: [u8; 4096] = [0; 4096];
        let write_res = connection.stream().write_all(&write_data);
        if write_res.is_err() {
            println!("connection.write error");
            return None;
        }
        let write_res = connection.stream().flush();
        if write_res.is_err() {
            println!("connection.flush error");
            return None;
        }
        println!("connection.write success");

        // 2-3. read a signed_cert
        //
        // taint analysis PoC with RA-TLS:
        // -- (1) tag read_signed_cert as UnencryptedSecret --> MIRAI!
        // -- (2) use 'mirai_test' as an undesirable destination for tagged data and see if MIRAI can detect this!
        // -- MIRAI issue: https://github.com/facebookexperimental/MIRAI/issues/508
        let mut read_signed_cert: [u8; 4096] = [0; 4096];
        let read_res = connection.stream().read_exact(&mut read_signed_cert);
        match read_res {
            Err(e) => {
                println!("connection.read error: {}", e);
                return None;
            },
            Ok(_) => {
                println!("connection.read success");
            },
        }

        // 2-4. verify the signed cert
        // TODO: verify

        // 3. write the signed cert to "shared memory"
        //let data: [u8; 4096] = [1; 4096];
        let write_res = self.shared_memory.write_only(self.id, &read_signed_cert);
        if write_res == false {
            println!("shared_memory.write_only error");
            return None;
        }

        // 4. generate a remote channel stream
        // [TODO] a wrapper struct that offers security to "TcpStream"
        // in order to ensure that the created TcpStream will not change after this flow..
        // --> how to guarantee this? --> prusti check! (possible?)
        match RemoteChannel::connect() {
            Some(rc) => {
                Some((
                    Gateway {
                        id: self.id,
                        shared_memory: self.shared_memory.into_read_write(),
                        state: Established,
                        rc_state: Established,
                    },
                    rc,
                ))
            },
            None => {
                println!("RemoteChannel::connect() error");
                None
            },
        }
    }

    pub fn test_mirai_taint(self) -> Data<UnencryptedRemoteData> {
        let read_signed_cert: [u8; 4096] = [0; 4096];
        let signed_cert_data = Data {
            data: read_signed_cert,
            state: UnencryptedRemoteData,
        };
        //add_tag!(&signed_cert_data, Sanitized);
        add_tag!(&signed_cert_data, Tainted);
        signed_cert_data
    }
}

#[pure]
fn ensures_read_from_remote(res: &Option<Data<EncryptedRemoteData>>) -> bool {
    match res {
        Some(d) => {
            d.state.is_encrypted_remote_data()
        },
        None => true,
    }
}
#[pure]
fn ensures_read_from_local(res: &Option<Data<UnencryptedLocalData>>) -> bool {
    match res {
        Some(d) => {
            d.state.is_unencrypted_local_data()
        },
        None => true,
    }
}

impl Gateway<Established, ReadWrite, Established> {
    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) && ((&self).rc_state.is_rc_established()) )]
    pub fn perform(&self) {
        //println!("perform! {}", self.id);
    }

    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) && ((&self).rc_state.is_rc_established()) )]
    #[ensures(ensures_read_from_local(&result))]
    pub fn read_from_local(&self) -> Option<Data<UnencryptedLocalData>> {
        self.shared_memory.read(self.id)
    }

    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) && ((&self).rc_state.is_rc_established()) )]
    #[ensures(ensures_read_from_remote(&result))]
    pub fn read_from_remote(&self, rc: &mut RemoteChannel) -> Option<Data<EncryptedRemoteData>> {
        let mut data: [u8; 4096] = [0; 4096];
        match rc.stream.read_exact(&mut data) {
            Ok(_) => {
                Some(Data::<Uninitialized>::new_from_remote(data))
            },
            Err(_) => {
                None
            },
        }
    }

    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) && ((&self).rc_state.is_rc_established()) && (data.state.is_unencrypted_remote_data()) )]
    pub fn write_to_local(&self, data: &Data<UnencryptedRemoteData>) -> bool {
        self.shared_memory.write(self.id, data)
    }

    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) && ((&self).rc_state.is_rc_established()) && (data.state.is_unencrypted_remote_data()) )]
    pub fn write_to_remote(&self, rc: &mut RemoteChannel, data: Data<EncryptedLocalData>) -> bool {
        // [TODO] security: wipe out data after write- how? (zeroize trait?)
        match rc.stream.write(&data.data) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) && ((&self).rc_state.is_rc_established()) && (data.state.is_unencrypted_local_data()) )]
    #[ensures((&result).state.is_encrypted_local_data())]
    pub fn encrypt_data(&self, data: Data<UnencryptedLocalData>) -> Data<EncryptedLocalData> {
        data.encrypt()
    }

    #[requires( ((&self).state.is_established()) && ((&self).shared_memory.state.is_read_write()) && ((&self).rc_state.is_rc_established()) && (data.state.is_encrypted_remote_data()) )]
    #[ensures((&result).state.is_unencrypted_remote_data())]
    pub fn decrypt_data(&self, data: Data<EncryptedRemoteData>) -> Data<UnencryptedRemoteData> {
        data.decrypt()
    }
}

// Typestate definitions for RemoteChannelState
// Initialized --> Established
pub trait RemoteChannelState {
    #[pure]
    fn is_rc_initialized(&self) -> bool { false }

    #[pure]
    fn is_rc_established(&self) -> bool { false }
}

#[refine_trait_spec]
impl RemoteChannelState for Initialized {
    #[pure]
    #[ensures(result == true)]
    fn is_rc_initialized(&self) -> bool { true }
}

#[refine_trait_spec]
impl RemoteChannelState for Established {
    #[pure]
    #[ensures(result == true)]
    fn is_rc_established(&self) -> bool { true }
}

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

// Typestate definitions for DataState
pub struct Uninitialized;
pub struct UnencryptedLocalData;
pub struct EncryptedLocalData;
pub struct EncryptedRemoteData;
pub struct UnencryptedRemoteData;

pub trait DataState {
    #[pure]
    fn is_uninitialized(&self) -> bool { false }

    #[pure]
    fn is_unencrypted_local_data(&self) -> bool { false }

    #[pure]
    fn is_encrypted_local_data(&self) -> bool { false }

    #[pure]
    fn is_unencrypted_remote_data(&self) -> bool { false }

    #[pure]
    fn is_encrypted_remote_data(&self) -> bool { false }
}

#[refine_trait_spec]
impl DataState for Uninitialized {
    #[pure]
    #[ensures(result == true)]
    fn is_uninitialized(&self) -> bool { true }
}

#[refine_trait_spec]
impl DataState for UnencryptedLocalData {
    #[pure]
    #[ensures(result == true)]
    fn is_unencrypted_local_data(&self) -> bool { true }
}

#[refine_trait_spec]
impl DataState for EncryptedLocalData {
    #[pure]
    #[ensures(result == true)]
    fn is_encrypted_local_data(&self) -> bool { true }
}

#[refine_trait_spec]
impl DataState for EncryptedRemoteData {
    #[pure]
    #[ensures(result == true)]
    fn is_encrypted_remote_data(&self) -> bool { true }
}

#[refine_trait_spec]
impl DataState for UnencryptedRemoteData {
    #[pure]
    #[ensures(result == true)]
    fn is_unencrypted_remote_data(&self) -> bool { true }
}

// test function for MIRAI
pub fn remote_channel_sink_test<S: DataState>(data: Data<S>) {
    precondition!(does_not_have_tag!(&data, Tainted) || has_tag!(&data, Sanitized));
    println!("hi");
}

pub fn test_mirai_taint() -> Data<UnencryptedRemoteData> {
    let read_signed_cert: [u8; 4096] = [1; 4096];
    let signed_cert_data = Data {
        data: read_signed_cert,
        state: UnencryptedRemoteData,
    };
    //add_tag!(&signed_cert_data, Sanitized);
    add_tag!(&signed_cert_data, Tainted);
    signed_cert_data
}