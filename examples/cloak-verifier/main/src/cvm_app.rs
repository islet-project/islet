#[macro_use]
extern crate mirai_annotations;

use local_channel_app::app::{LocalChannelApp, Start, Unmapped as UnmappedApp, Established, ReadWrite};
use gateway::app::{Gateway, Initialized, Unmapped as UnmappedGateway, test_mirai_taint, remote_channel_sink_test};
use common::shared::*;
use common::util::*;
use std::env;
use std::io::{self, BufRead};
use std::fs::File;
use std::os::fd::AsRawFd;

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

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

#[cfg(target_arch = "aarch64")]
fn do_open_syscall(_ch_app: &LocalChannelApp<Established, ReadWrite>) -> i32 {
    let mut file = match File::create("/shared/hello.txt") {
        Err(_) => {
            println!("file open error!");
            -1
        }
        Ok(f) => {
            println!("raw_fd: {}", f.as_raw_fd());
            f.as_raw_fd()
        },
    };
}
#[cfg(any(target_arch = "x86_64", target_arch = "emul"))]
fn do_open_syscall(ch_app: &LocalChannelApp<Established, ReadWrite>) -> i32 {
    let name = String::from("hello_test.txt");
    let mut open_req = SharedOpenReq {
        dtype: DEL_SYSCALL_OPEN,
        filename: [0; 256],
        flags: 0,
        mode: 0,
    };
    for (i, ch) in name.chars().enumerate() {
        open_req.filename[i] = ch as u8;
    }

    let open_req_bytes = unsafe { any_as_u8_slice(&open_req) };
    let mut write_data: [u8; 4096] = [0; 4096];
    let mut read_data: [u8; 4096] = [0; 4096];
    for (dst, src) in write_data.iter_mut().zip(open_req_bytes) {
        *dst = *src;
    }

    // 1. open()
    match ch_app.write(&write_data) {
        true => {},
        false => {
            println!("channel_app.write failed");
            return -1;
        },
    }
    println!("CVM_app: open() request sent. wait..");
    get_line();

    match ch_app.read(&mut read_data) {
        true => {},
        false => {
            println!("channel_app.read failed");
            return -1;
        },
    }
    let mut buf: [u8; 4] = [0; 4];
    buf.copy_from_slice(&read_data[8..12]); 
    let fd = i32::from_le_bytes(buf);
    println!("CVM_app: open() success: fd {}", fd);
    fd
}

#[cfg(target_arch = "aarch64")]
fn do_write_syscall(_ch_app: &LocalChannelApp<Established, ReadWrite>) -> u32 {
    0
}
#[cfg(any(target_arch = "x86_64", target_arch = "emul"))]
fn do_write_syscall(ch_app: &LocalChannelApp<Established, ReadWrite>, fd: i32) -> u32 {
    let write_req = SharedWriteReq {
        dtype: DEL_SYSCALL_WRITE,
        fd: fd,
        size: 16,
        data: [7; 2048],
    };

    let req_bytes = unsafe { any_as_u8_slice(&write_req) };
    let mut write_data: [u8; 4096] = [0; 4096];
    let mut read_data: [u8; 4096] = [0; 4096];
    for (dst, src) in write_data.iter_mut().zip(req_bytes) {
        *dst = *src;
    }

    // 1. write()
    match ch_app.write(&write_data) {
        true => {},
        false => {
            println!("channel_app.write failed");
            return 0;
        },
    }
    println!("CVM_app: write() request sent. wait..");
    get_line();

    match ch_app.read(&mut read_data) {
        true => {},
        false => {
            println!("channel_app.read failed");
            return 0;
        },
    }
    let mut buf: [u8; 4] = [0; 4];
    buf.copy_from_slice(&read_data[8..12]); 
    let write_size = u32::from_le_bytes(buf);
    println!("write size: {}", write_size);
    write_size
}

#[cfg(target_arch = "aarch64")]
fn do_read_syscall(_ch_app: &LocalChannelApp<Established, ReadWrite>, _fd: i32, _data: &mut [u8; 16]) -> u32 {
    0
}
#[cfg(any(target_arch = "x86_64", target_arch = "emul"))]
fn do_read_syscall(ch_app: &LocalChannelApp<Established, ReadWrite>, fd: i32, data: &mut [u8; 16]) -> u32 {
    let read_req = SharedReadReq {
        dtype: DEL_SYSCALL_READ,
        fd: fd,
        size: 16,
    };

    let req_bytes = unsafe { any_as_u8_slice(&read_req) };
    let mut write_data: [u8; 4096] = [0; 4096];
    let mut read_data: [u8; 4096] = [0; 4096];
    for (dst, src) in write_data.iter_mut().zip(req_bytes) {
        *dst = *src;
    }

    // 1. read()
    match ch_app.write(&write_data) {
        true => {},
        false => {
            println!("channel_app.write failed");
            return 0;
        },
    }
    println!("CVM_app: read() request sent. wait..");
    get_line();

    match ch_app.read(&mut read_data) {
        true => {},
        false => {
            println!("channel_app.read failed");
            return 0;
        },
    }

    let read_resp: SharedReadResp = get_shared_type(&read_data);
    let read_size = read_resp.size;
    println!("read size: {} read data: {}", read_size, read_resp.data[0]);
    read_size
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

    /*
    // 2. APP: wait_for_signed_cert
    println!("type in anything after CVM_GW writes signed_cert to shared memory..");
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).unwrap();

    let channel_app = channel_app.unwrap().wait_for_signed_cert();
    println!("channel_app.wait_for_signed_cert success");

    // 3. APP: establish
    let channel_app = channel_app.establish();
    if channel_app.is_none() {
        println!("channel_app.establish error");
        return;
    }
    println!("channel_app.establish success"); */

    let channel_app = channel_app.unwrap();

    // 2. read some data through remote channel
    /*
    let mut read_data: [u8; 4096] = [0; 4096];
    match channel_app.read(&mut read_data) {
        true => {
            println!("channel_app.read success: {:x}", read_data[0]);
        },
        false => {
            println!("channel_app.read failed");
            return;
        },
    } */

    // 3. storage test: open()-write()-read() system call
    let mut read_data: [u8; 16] = [0; 16];
    let fd = do_open_syscall(&channel_app);
    let _ = do_write_syscall(&channel_app, fd);
    let _ = do_read_syscall(&channel_app, fd, &mut read_data);

    // 4. transmit some data through remote channel
    // App write something to LocalChannel
    /*
    let write_data: [u8; 4096] = [3; 4096];
    match channel_app.write(&write_data) {
        true => {},
        false => {
            println!("channel_app.write failed");
            return;
        },
    } */

    println!("CVM_App end");
}
