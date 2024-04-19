use std::io;
use std::io::BufRead;

pub fn get_line() {
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).unwrap();
}