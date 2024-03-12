extern crate serde;
extern crate serde_json;

//use serde::{Serialize, Deserialize};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use rand::rngs::OsRng;
use std::fs::File;
use std::io::{Read, Write};

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let key_size = 2048;

    println!("RsaPrivateKey::new start");
    let private_key = RsaPrivateKey::new(&mut OsRng, key_size)?;
    println!("RsaPrivateKey::new end");

    // save
    let mut f = File::create("priv_key.serde.arm64")?;
    let write_buf = serde_json::to_vec(&private_key)?;
    f.write_all(&write_buf[..])?;

    // load test
    if let Ok(mut file) = File::open("priv_key.serde.arm64") {
        let mut buf = vec![];
        if file.read_to_end(&mut buf).is_ok() {
            if let Ok(loaded_key) = serde_json::from_slice(&buf[..]) {
                if private_key == loaded_key {
                    println!("load test success");
                } else {
                    println!("load test fail");
                }

                let pub_key = private_key.to_public_key();
                let data = b"hello world";
                let enc_data = pub_key.encrypt(&mut OsRng, Pkcs1v15Encrypt, &data[..]).expect("failed to encrypt");
                let dec_data = private_key.decrypt(Pkcs1v15Encrypt, &enc_data).expect("failed to decrypt");
                assert_eq!(&data[..], &dec_data[..]);
                println!("correct private key!");
            }
        }
    }

    Ok(())
}
