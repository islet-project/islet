use common::shared::*;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::raw::{c_int, c_ulong};
use std::os::fd::AsRawFd;
use std::os::raw::c_void;
use std::collections::HashMap;

pub fn local_op_storage_open(data: SharedOpenReq, storage_map: &mut HashMap<i32, File>) -> io::Result<SharedOpenResp> {
    let mut v = Vec::new();
    for c in data.filename.iter() {
        if *c == 0 {
            break;
        }
        v.push(*c);
    }

    let fname = std::str::from_utf8(v.as_slice()).expect("utf8 error");
    let file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .read(true)
                        .open(fname)?;
    let fd = file.as_raw_fd();
    storage_map.insert(fd, file);

    Ok(SharedOpenResp {
        dtype: DEL_SYSCALL_OPEN,
        magic_num: DEL_OPEN_MAGIC_NUM,
        fd: fd,
    })
}

pub fn local_op_storage_write(data: SharedWriteReq, storage_map: &mut HashMap<i32, File>) -> io::Result<SharedWriteResp> {
    let fd = data.fd;
    let count = data.size as usize;

    println!("storage_write: {}-{}-{}", fd, count, data.data[0]);

    match storage_map.get_mut(&fd) {
        Some(file) => {
            let mut v = Vec::new();
            for (i, ch) in data.data.iter().enumerate() {
                if i >= count {
                    break;
                }
                v.push(*ch);
            }
            let res = file.write(v.as_slice())?;
            file.flush()?;
            file.sync_all()?;
            println!("storage_write success");

            Ok(SharedWriteResp {
                dtype: DEL_SYSCALL_WRITE,
                magic_num: DEL_WRITE_MAGIC_NUM,
                size: res as u32,
            })
        },
        None => {
            println!("storage_write error!");

            Ok(SharedWriteResp {
                dtype: DEL_SYSCALL_WRITE,
                magic_num: DEL_WRITE_MAGIC_NUM,
                size: 0 as u32,
            })
        },
    }
}

pub fn local_op_storage_read(data: SharedReadReq, storage_map: &mut HashMap<i32, File>) -> io::Result<SharedReadResp> {
    let fd = data.fd;
    let count = data.size as usize;

    println!("storage_read: {}-{}", fd, count);

    match storage_map.get_mut(&fd) {
        Some(file) => {
            let mut read_data: [u8; 2048] = [0; 2048];
            let res = file.read(&mut read_data)?;
            //file.read_exact(&mut read_data)?;
            println!("storage_read success: size: {}, data: {}", res, read_data[0]);

            Ok(SharedReadResp {
                dtype: DEL_SYSCALL_READ,
                magic_num: DEL_READ_MAGIC_NUM,
                size: res as u32,
                data: read_data,
            })
        },
        None => {
            println!("storage_read error!");

            Ok(SharedReadResp {
                dtype: DEL_SYSCALL_READ,
                magic_num: DEL_READ_MAGIC_NUM,
                size: 0 as u32,
                data: [0; 2048],
            })
        },
    }
}
