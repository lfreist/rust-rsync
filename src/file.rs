use std::fs;
use std::fs::DirEntry;
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::vec::IntoIter;
use crate::checksum;

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

pub struct DataChunk {
    pub begin: usize,
    pub size: usize,
    pub adler32: u32,
    pub md5: Option<String>,
}


struct RecursiveDirectoryIterator {
    stack: Vec<IntoIter<DirEntry>>,
}

impl RecursiveDirectoryIterator {
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let root_iter = fs::read_dir(path)?
            .collect::<Result<Vec<_>, io::Error>>()?
            .into_iter();
        Ok(Self { stack: vec![root_iter] })
    }
}

impl Iterator for RecursiveDirectoryIterator {
    type Item = io::Result<PathBuf>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(iter) = self.stack.last_mut() {
            match iter.next() {
                Some(entry) => {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Ok(sub_iter) = fs::read_dir(&path) {
                            let sub_iter = match sub_iter.collect::<Result<Vec<_>, io::Error>>() {
                                Ok(iter) => iter.into_iter(),
                                Err(e) => return Some(Err(e)),
                            };
                            self.stack.push(sub_iter);
                        }
                    } else {
                        return Some(Ok(path));
                    }
                }
                None => {
                    self.stack.pop();
                }
            }
        }
        None
    }
}

pub struct FileChunkIterator<R> {
    reader: R,
    chunk_size: usize,
    rolling: bool,
    offset: usize,
}

impl<R: Read + Seek> FileChunkIterator<R> {
    pub fn new(reader: R, chunk_size: usize, rolling: bool) -> Self {
        Self {
            reader,
            chunk_size,
            rolling,
            offset: 0,
        }
    }
}

impl<R: Read + Seek> Iterator for FileChunkIterator<R> {
    type Item = io::Result<DataChunk>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rolling {
            let mut buffer = vec![0; self.chunk_size];
            match self.reader.seek(SeekFrom::Start(self.offset as u64)) {
                Ok(_) => {
                    let mut buffer = vec![0; self.chunk_size];
                    match self.reader.read(&mut buffer) {
                        Ok(bytes_read) => {
                            if bytes_read == 0 {
                                None
                            } else {
                                let adler32_checksum = checksum::adler32(&buffer);
                                let chunk = DataChunk {
                                    begin: self.offset,
                                    size: bytes_read,
                                    adler32: adler32_checksum,
                                    md5: None,
                                };
                                self.offset += 1;
                                Some(Ok(chunk))
                            }
                        }
                        Err(e) => Some(Err(e)),
                    }
                }
                Err(e) => Some(Err(e)),
            }
        } else {
            let mut buffer = vec![0; self.chunk_size];
            match self.reader.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        None
                    } else {
                        let adler32_checksum = checksum::adler32(&buffer);
                        let md5_checksum = Some(checksum::md5(&buffer));
                        let chunk = DataChunk {
                            begin: self.offset,
                            size: bytes_read,
                            adler32: adler32_checksum,
                            md5: md5_checksum,
                        };


                        self.offset += bytes_read;
                        Some(Ok(chunk))
                    }
                }
                Err(e) => Some(Err(e)),
            }
        }
    }
}