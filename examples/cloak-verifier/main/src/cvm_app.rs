#[macro_use]
extern crate mirai_annotations;

use local_channel_app::app::{LocalChannelApp, Start, Unmapped as UnmappedApp};
use gateway::app::{Gateway, Initialized, Unmapped as UnmappedGateway, test_mirai_taint, remote_channel_sink_test};
use std::env;
use std::io::{self, BufRead};

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
    // args[1] == id
    let mut channel_id = 0;
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("args[1]: id");
        return;
    }
    if let Some(id_str) = args.get(1) {
        channel_id = id_str.trim().parse::<usize>().expect("type a number!");
        println!("app channel_id: {}", channel_id);
    }
    println!("CVM_App start");

    // 1. APP: connect
    let channel_app = LocalChannelApp::<Start, UnmappedApp>::new(channel_id);
    let channel_app = channel_app.connect();
    if channel_app.is_none() {
        println!("channel_app.connect error");
        return;
    }
    println!("channel_app.connect success");

    // 2. APP: wait_for_signed_cert
    let channel_app = channel_app.unwrap().wait_for_signed_cert();
    println!("channel_app.wait_for_signed_cert success");

    // 3. APP: establish
    let channel_app = channel_app.establish();
    if channel_app.is_none() {
        println!("channel_app.establish error");
        return;
    }
    println!("channel_app.establish success");

    let channel_app = channel_app.unwrap();

    // 4. transmit some data through remote channel
    // App write something to LocalChannel
    let write_data: [u8; 4096] = [3; 4096];
    match channel_app.write(&write_data) {
        true => {},
        false => {
            println!("channel_app.write failed");
            return;
        },
    }

    println!("CVM_App end");
}
