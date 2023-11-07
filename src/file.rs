use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;

pub struct File {
    file: fs::File,
    block_size: usize,
}

impl File {
    pub fn new<P: AsRef<Path>>(path: P, block_size: usize, write: bool) -> io::Result<Self> {
        let file = fs::File::options()
            .read(true)
            .write(write)
            .open(path.as_ref())
            .expect("Could not open file {path}");
        Ok(File { file, block_size })
    }

    pub fn get_next_block(&mut self) -> io::Result<Option<String>> {
        let mut buffer = vec![0; self.block_size];
        let bytes_read = self.file.read(&mut buffer)?;
        if bytes_read == 0 {
            return Ok(None);
        }
        let block = String::from_utf8_lossy(&buffer[..bytes_read]);
        Ok(Some(block.to_string()))
    }
}

impl Iterator for File {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.get_next_block() {
            Ok(Some(block)) => Some(Ok(block)),
            Ok(None) => None,
            Err(e) => Some(Err(e))
        }
    }
}