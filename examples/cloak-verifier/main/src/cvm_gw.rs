#[macro_use]
extern crate mirai_annotations;

use local_channel_app::app::{LocalChannelApp, Start, Unmapped as UnmappedApp};
use gateway::app::{Gateway, Initialized, Unmapped as UnmappedGateway, test_mirai_taint, remote_channel_sink_test};
use std::env;

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

fn main() {
    // args[1] == id, args[2] == mode (server or client)
    let mut mode_server = false;
    let mut channel_id = 0;

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("args[1]: id, args[2]: mode (server or client)");
        return;
    }
    if let Some(id_str) = args.get(1) {
        println!("id_str: {}", id_str);
        if let Ok(id) = id_str.trim().parse::<usize>() {
            channel_id = id;
        }
        println!("gateway channel_id: {}", channel_id);
    }
    if let Some(mode) = args.get(2) {
        if mode == "server" {
            mode_server = true;
        }
        println!("gateway mode_server: {}", mode_server);
    }

    println!("CVM_Gateway start");

    // 1. GW: create
    let channel_gw = Gateway::<Initialized, UnmappedGateway, Initialized>::new(channel_id, mode_server);
    let channel_gw = channel_gw.create();
    if channel_gw.is_none() {
        println!("channel_gw.create error");
        return;
    }
    println!("channel_gw.create success");

    // 2. GW: wait_for_app
    let channel_gw = channel_gw.unwrap().wait_for_app();
    println!("channel_gw.wait_for_app success");

    // test for MIRAI
    //let signed_data = channel_gw.test_mirai_taint();
    //add_tag!(&signed_data, Sanitized);
    //remote_channel_sink_test(signed_data);

    // 3. GW: establish
    let gw_establish_res = channel_gw.establish();
    if gw_establish_res.is_none() {
        println!("channel_gw.establish error");
        return;
    }
    let (channel_gw, mut remote_channel) = gw_establish_res.unwrap();
    println!("channel_gw.establish success");

    // 4. GW reads from LocalChannel
    let local_data = channel_gw.read_from_local();
    if local_data.is_none() {
        println!("channel_gw.read_from_local failed");
        return;
    }
    let local_data = local_data.unwrap();
    let local_enc_data = channel_gw.encrypt_data(local_data);

    // 5. GW read or write
    if mode_server {
        // server mode (read first)
        channel_gw.run_server(&mut remote_channel);
    } else {
        // client mode (write first)
        match channel_gw.write_to_remote(&mut remote_channel, local_enc_data) {
            true => println!("write_to_remote success!"),
            false => println!("write_to_remote failed!"),
        }
    }

    println!("CVM_Gateway end");
}
