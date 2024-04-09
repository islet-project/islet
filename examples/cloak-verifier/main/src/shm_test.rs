use common::ioctl::{cloak_create, cloak_connect, cloak_write, cloak_read, cloak_status};
use std::io::{self, BufRead};
use std::env;

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
        println!("mode_server: {}", mode_server);
    }

    if mode_server {
        // channel creator
        let res = cloak_create(channel_id);
        if res.is_err() {
            println!("cloak_create error");
            return;
        }

        println!("type in anything after channel write..");
        let mut line = String::new();
        io::stdin().lock().read_line(&mut line).unwrap();

        let mut data: [u8; 4096] = [0; 4096];
        let res = cloak_read(channel_id, &mut data);
        if res.is_err() {
            println!("cloak_read error");
            return;
        }
        println!("cloak_read success: {}", data[0]);
    } else {
        // channel connector
        let res = cloak_connect(channel_id);
        if res.is_err() {
            println!("cloak_connect error");
            return;
        }

        let data: [u8; 4096] = [7; 4096];
        let mut read_data: [u8; 4096] = [0; 4096];
        let res = cloak_write(channel_id, &data);
        if res.is_err() {
            println!("cloak_write error");
            return;
        }
        println!("cloak_write success");

        let res = cloak_read(channel_id, &mut read_data);
        if res.is_err() {
            println!("cloak_read error");
            return;
        }
        println!("cloak_read success: {}", read_data[0]);
    }
}