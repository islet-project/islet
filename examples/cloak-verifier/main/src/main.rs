use local_channel_app::app::{LocalChannelApp, Start, Unmapped as UnmappedApp};
use gateway::app::{Gateway, Initialized, Unmapped as UnmappedGateway};

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

fn test_simple() {
    println!("test_simple start");

    // 1. GW: create
    let channel_gw = Gateway::<Initialized, UnmappedGateway, Initialized>::new();
    let channel_gw = channel_gw.create();
    if channel_gw.is_none() {
        println!("channel_gw.create error");
        return;
    }
    println!("channel_gw.create success");

    // 2. APP: connect
    let channel_app = LocalChannelApp::<Start, UnmappedApp>::new();
    let channel_app = channel_app.connect();
    if channel_app.is_none() {
        println!("channel_app.connect error");
        return;
    }
    println!("channel_app.connect success");

    // 3. GW: wait_for_app
    let channel_gw = channel_gw.unwrap().wait_for_app();
    println!("channel_gw.wait_for_app success");

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

fn main() {
    //test_local_channel_app();
    //test_local_channel_gateway();
    //test_all();
    test_simple();
}
