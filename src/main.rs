
extern crate rsync;

use std::env;
use rsync::file::{FileChunkIterator};
use std::fs::File;

fn main() {
    let args: Vec<String> = env::args().collect();
    simple_logger::SimpleLogger::new()
        .init()
        .unwrap();
    /*
    let file = rsync::file::File::new("Cargo.toml", 32, false)?;
    for (index, chunk_result) in file.enumerate() {
        match chunk_result {
            Ok(chunk) => {
                println!("Chunk {index}:");
                println!("{chunk}");
                println!("Adler32: {}", rsync::checksum::adler32(&chunk));
                println!("Md5    : {:x}", rsync::checksum::md5(&chunk));
            },
            Err(e) => {
                eprintln!("Error reading chunk {index}: {e}");
                return Err(e);
            }
        }
    }
    */
    let file = File::open(&args[1]).unwrap();
    let iterator = FileChunkIterator::new(file, 128, true);

    for chunk_result in iterator {
        match chunk_result {
            Ok(chunk) => {
                println!("Chunk at offset {}: size={} adler32={} md5={:?}",
                         chunk.begin, chunk.size, chunk.adler32, chunk.md5);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}
