use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::Read;
use std::io::Write;
use core::result::Result::{Ok, Err};
use core::panic;

fn handle_client(mut stream: TcpStream) {
    // read 20 bytes at a time from stream echoing back to stream
    loop {
        let mut read: [u8; 4096] = [0; 4096];
        let write: [u8; 4096] = [1; 4096];
        match stream.read_exact(&mut read) {
            Ok(_) => {
                if stream.write(&write).is_err() {
                    println!("failed to stream.write()")
                }
                println!("read: {:x}, echo success", read[0]);  // GIT?
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }
}

fn main() {
    println!("server start..");

    let listener = TcpListener::bind("193.168.10.15:1999").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream);
                });
            }
            Err(_) => {
                println!("Error");
            }
        }
    }
}