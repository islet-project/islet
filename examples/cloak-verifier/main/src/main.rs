#[macro_use]
extern crate mirai_annotations;

use local_channel_app::app::{LocalChannelApp, Start, Unmapped as UnmappedApp};
use gateway::app::{Gateway, Initialized, Unmapped as UnmappedGateway, test_mirai_taint, remote_channel_sink_test};

// for testing MIRAI
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

#[derive(Clone, Copy)]
pub struct Data<S> {
    data: [u8; 4096],
    state: S,
}
pub struct None;
pub struct Unencrypted;
pub struct Encrypted;

pub trait DataState {
    fn dummy(&self) -> bool { true }
}
impl DataState for None {
    fn dummy(&self) -> bool { true }
}
impl DataState for Unencrypted {
    fn dummy(&self) -> bool { true }
}
impl DataState for Encrypted {
    fn dummy(&self) -> bool { true }
}

impl<S: DataState> Data<S> {
    pub fn read() -> Data<Unencrypted> {
        let data: [u8; 4096] = [0; 4096];
        let d = Data {
            data: data,
            state: Unencrypted,
        };
        add_tag!(&d, Tainted);
        d
    }
}
impl Data<Unencrypted> {
    pub fn encrypt(self) -> Data<Encrypted> {
        let d = Data {
            data: self.data,
            state: Encrypted
        };
        add_tag!(&d, Sanitized);
        d
    }
}

fn sink_func<S: DataState>(data: Data<S>) {
    precondition!(does_not_have_tag!(&data, Tainted) || has_tag!(&data, Sanitized));
    println!("hi");
}

/*
fn test_local_channel_app() {
    let channel = LocalChannelApp::<Start, UnmappedApp>::new();
    let channel = channel.connect();
    let channel = channel.wait_for_signed_cert();
    let channel = channel.establish();
    channel.perform();
}

fn test_local_channel_gateway() {
    let channel = Gateway::<Initialized, UnmappedGateway>::new();
    if let Some(channel) = channel.create() {
        let channel = channel.wait_for_app();
        let channel = channel.establish();
        channel.perform();
    }
    else {
        return;
    }  
} */

/*
fn test_all() {
    // 1. GW: create
    let channel_gw = Gateway::<Initialized, UnmappedGateway>::new();
    let channel_gw = channel_gw.create();
    if channel_gw.is_none() {
        return;
    }

    // 2. APP: connect
    let channel_app = LocalChannelApp::<Start, UnmappedApp>::new();
    let channel_app = channel_app.connect();

    // 3. GW: wait_for_app
    let channel_gw = channel_gw.unwrap().wait_for_app();

    // 4. APP: wait_for_signed_cert
    let channel_app = channel_app.wait_for_signed_cert();

    // 5. GW: establish
    let channel_gw = channel_gw.establish();

    // 6. APP: establish
    let channel_app = channel_app.establish();

    channel_gw.perform();
    channel_app.perform();
} */

fn test_mirai() {
    //let d: [u8; 4096] = [0; 4096];
    let data = Data::<None>::read();
    let data = data.encrypt();  // when it's commented out, MIRAI reported unsatisifed condition! (this matches my expectation)
    sink_func(data);
}

fn test_data_mirai() {
    // test for MIRAI
    let signed_data = test_mirai_taint();
    //add_tag!(&signed_data, Sanitized);
    remote_channel_sink_test(signed_data);
}

fn main() {
    //test_local_channel_app();
    //test_local_channel_gateway();
    //test_all();
    
    // test
    //println!("test_simple start");

    // 1. GW: create
    let channel_gw = Gateway::<Initialized, UnmappedGateway, Initialized>::new();
    let channel_gw = channel_gw.create();
    if channel_gw.is_none() {
        //println!("channel_gw.create error");
        return;
    }
    //println!("channel_gw.create success");

    // 2. APP: connect
    let channel_app = LocalChannelApp::<Start, UnmappedApp>::new();
    let channel_app = channel_app.connect();
    if channel_app.is_none() {
        //println!("channel_app.connect error");
        return;
    }
    //println!("channel_app.connect success");

    // 3. GW: wait_for_app
    let channel_gw = channel_gw.unwrap().wait_for_app();
    //println!("channel_gw.wait_for_app success");

    // test for MIRAI
    //let signed_data = channel_gw.test_mirai_taint();
    //add_tag!(&signed_data, Sanitized);
    //remote_channel_sink_test(signed_data);

    // 4. GW: establish
    let gw_establish_res = channel_gw.establish();
    if gw_establish_res.is_none() {
        println!("channel_gw.establish error");
        return;
    }
    let (channel_gw, mut remote_channel) = gw_establish_res.unwrap();
    println!("channel_gw.establish success");

    // 5. APP: wait_for_signed_cert
    let channel_app = channel_app.unwrap().wait_for_signed_cert();
    println!("channel_app.wait_for_signed_cert success");

    // 6. APP: establish
    let channel_app = channel_app.establish();
    if channel_app.is_none() {
        println!("channel_app.establish error");
        return;
    }
    println!("channel_app.establish success");

    let channel_app = channel_app.unwrap();

    // 7. transmit some data through remote channel
    // 7-1. App write something to LocalChannel
    let write_data: [u8; 4096] = [3; 4096];
    match channel_app.write(&write_data) {
        true => {},
        false => {
            println!("channel_app.write failed");
            return;
        },
    }

    // 7-2. GW reads from LocalChannel
    let local_data = channel_gw.read_from_local();
    if local_data.is_none() {
        println!("channel_gw.read_from_local failed");
        return;
    }
    let local_data = local_data.unwrap();
    let local_enc_data = channel_gw.encrypt_data(local_data);

    // 7-3. GW writes it to RemoteChannel
    match channel_gw.write_to_remote(&mut remote_channel, local_enc_data) {
        true => println!("write_to_remote success!"),
        false => println!("write_to_remote failed!"),
    }

    println!("test_simple end");
}
