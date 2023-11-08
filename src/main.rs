
extern crate rsync;

use rsync::client::Verbosity;

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
    let client = rsync::client::RsyncClient::new(Verbosity::HIGH);
    let result = client.run("test".as_ref(), "test2".as_ref(), None);
    if result {
        println!("Success!");
    } else {
        println!("Failed!");
    }
}
