
extern crate rsync;

use std::thread;
use rsync::client::Client;
use rsync::server::Server;

fn main() {
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
    let handle = thread::spawn(|| {
        let server = rsync::server::UdpServer::new("127.0.0.1:6666");
        server.run();
    });
    let client = rsync::client::UdpClient::new("127.0.0.1:6666");
    let mut data = String::new();
    data.push_str("test data");
    client.send_receive(&data).expect("TODO: panic message");
    handle.join().unwrap();
}
