mod ttp;
mod error;

fn main() {
    println!("server start..");
    let mut server = ttp::CVMServer::new();
    let _ = server.run();
}