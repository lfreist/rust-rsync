
extern crate rsync;

use std::env;
use std::fs::File;
use rsync::client::{Client};
use rsync::file::SlidingWindowReader;

fn main() {
    let args: Vec<String> = env::args().collect();
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap();

    let client = Client::new(args[1].parse().unwrap(), args[2].parse().unwrap(), 128);
    let success = client.run(None);
    println!("{}", success);
}
