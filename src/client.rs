use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::net::UdpSocket;
use std::io::Write;
use log::{info, debug, error};
use std::path::{Path, PathBuf, StripPrefixError};

use crate::{checksum, file};
use crate::file::{ChunkInfo, FileChunkIterator, RecursiveDirectoryIterator};

pub trait ClientTrait {
    fn send_receive(&self, data: &String) -> io::Result<String>;
}

pub struct TcpClient {
    address: String,
}

impl ClientTrait for TcpClient {
    fn send_receive(&self, data: &String) -> io::Result<String> {
        debug!(target: "TcpClient", "Connecting to server: {:?}", self.address);
        match TcpStream::connect(&self.address) {
            Ok(mut stream) => {
                info!(target: "TcpClient", "Sending data: {:?}", data);
                // If writing to the stream fails, return the error
                stream.write_all(data.as_bytes())?;

                let mut reader = BufReader::new(&stream);
                let mut response = Vec::new();
                // If reading from the stream fails, return the error
                reader.read_until(b'\0', &mut response)?;
                debug!(target: "TcpClient", "Answer received");
                // Convert the response to a String and return it
                Ok(String::from_utf8(response).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
            }
            Err(error) => {
                error!(target: "TcpClient", "Connection to {:?} failed: {:?}", self.address, error);
                // Return the error if the connection fails
                Err(error)
            }
        }
    }
}

impl TcpClient {
    pub fn new(address: &str) -> Self {
        TcpClient { address: address.to_string() }
    }
}

pub struct UdpClient {
    address: String,
}

impl ClientTrait for UdpClient {
    fn send_receive(&self, data: &String) -> io::Result<String> {
        debug!(target: "UdpClient", "Setting up Udp Connection: {:?}", self.address);
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        debug!(target: "UdpClient", "Sending package to {}", &self.address);
        socket.send_to(data.as_bytes(), &self.address)?;

        let mut response = [0; 1024];
        let (number_of_bytes, _src_addr) = socket.recv_from(&mut response)?;

        // Only convert the actual received bytes to a String
        let received_data = &response[..number_of_bytes];
        String::from_utf8(received_data.to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}


impl UdpClient {
    pub fn new(address: &str) -> Self {
        UdpClient {
            address: address.to_string(),
        }
    }
}

pub enum NetworkClient {
    UDP(UdpClient),
    TCP(TcpClient),
}

// Implement the ServerTrait for the Server enum
impl ClientTrait for NetworkClient {
    fn send_receive(&self, data: &String) -> io::Result<String> {
        match self {
            NetworkClient::UDP(udp_client) => udp_client.send_receive(data),
            NetworkClient::TCP(tcp_client) => tcp_client.send_receive(data),
        }
    }
}

pub enum Verbosity {
    OFF,
    LOW,
    REG,
    HIGH,
}

pub struct Sender {
    verbosity: Verbosity,
    block_size: usize,
    path: PathBuf,
}

struct FileInfo {
    path_suffix: PathBuf,
    data: HashMap<u32, HashMap<String, ChunkInfo>>,
}

struct Common {
    src_begin: usize,
    dst_begin: usize,
    size: usize,
}

struct FileCompare {
    path_suffix: PathBuf,
    common: Vec<Common>,
}

fn get_suffix(path: &Path, prefix: &Path) -> Result<PathBuf, StripPrefixError> {
    match path.strip_prefix(prefix) {
        Ok(suffix) => Ok(suffix.to_path_buf()),
        Err(e) => Err(e)
    }
}

fn merge_maps(fst: &mut HashMap<u32, HashMap<String, ChunkInfo>>, snd: HashMap<u32, HashMap<String, ChunkInfo>>) {
    for (key, value) in snd {
        match fst.get_mut(&key) {
            Some(map) => {
                map.extend(value);
            }
            None => {
                fst.insert(key, value);
            }
        }
    }
}

impl Sender {
    pub fn new(path: &Path, verbosity: Verbosity, block_size: usize) -> Sender {
        Sender { path: path.to_path_buf(), verbosity, block_size }
    }

    fn get_file_chunks(&self, file_path: &PathBuf) -> io::Result<FileInfo> {
        debug!(target: "RsyncClient.get_file_chunks", "Processing file {:?}", file_path);

        let file_result = File::open(&file_path);
        match file_result {
            Ok(file) => {
                let mut final_chunk_info_map: HashMap<u32, HashMap<String, ChunkInfo>> = HashMap::new();
                let reader = BufReader::new(file);
                let file_chunk_iterator = FileChunkIterator::new(reader, self.block_size);
                for chunk_result in file_chunk_iterator {
                    match chunk_result {
                        Ok(chunk_info_map) => merge_maps(&mut final_chunk_info_map, chunk_info_map),
                        Err(e) => eprintln!("Error: {}", e)
                    }
                }
                Ok(FileInfo {
                    path_suffix: get_suffix(file_path.as_path(), self.path.as_path()).unwrap(),
                    data: final_chunk_info_map,
                })
            }
            Err(e) => Err(e)
        }
    }
}

pub struct Receiver {
    path: PathBuf,
    verbosity: Verbosity,
}

impl Receiver {
    pub fn new(&mut self, path: &Path, verbosity: Verbosity) -> Self {
        Self { path: path.to_path_buf(), verbosity }
    }

    fn compare(&self, suffix: &Path, file_info: &FileInfo, chunk_size: usize) -> Option<FileCompare> {
        let file_path = if suffix.to_str().unwrap().is_empty() { self.path.clone() } else { self.path.join(suffix) };
        match File::open(&file_path) {
            Ok(file) => {
                let mut offset: u64 = 0;
                let mut matches: Vec<Common> = vec![];
                let mut window_shift = chunk_size;
                let mut reader = file::SlidingWindowReader::new(file, chunk_size);
                loop {
                    match reader.read(window_shift) {
                        Ok(data) => {
                            match data {
                                Some(data) => {
                                    let common = self.check_is_identical(&data.0, offset as usize, file_info);
                                    if common.is_some() {
                                        matches.push(common.unwrap());
                                        window_shift = chunk_size;
                                        offset += chunk_size as u64;
                                    } else {
                                        window_shift = 1;
                                        offset += 1;
                                    }
                                    if data.1 < chunk_size {
                                        break;
                                    }
                                }
                                None => break
                            }
                        }
                        Err(_) => break
                    }
                }
                Some(FileCompare { path_suffix: suffix.to_path_buf(), common: matches })
            }
            Err(_) => None
        }
    }

    fn check_is_identical(&self, slice: &[u8], offset: usize, file_info: &FileInfo) -> Option<Common> {
        match file_info.data.get(&checksum::adler32(slice)) {
            Some(md5_map) => {
                match md5_map.get(&checksum::md5(slice)) {
                    Some(source_chunk) => {
                        debug!(target: "Receiver.compare", "Common: {}, {}, size: {}", source_chunk.begin, offset, source_chunk.size);
                        Some(Common {
                            src_begin: source_chunk.begin,
                            dst_begin: offset,
                            size: source_chunk.size,
                        })
                    }
                    None => None
                }
            }
            None => None
        }
    }
}

pub struct Client {
    sender: Sender,
    receiver: Receiver,
    block_size: usize,
    src: PathBuf,
    dst: PathBuf,
}

impl Client {
    pub fn new(src: PathBuf, dst: PathBuf, block_size: usize) -> Self {
        Client {
            sender: Sender { verbosity: Verbosity::HIGH, block_size, path: src.clone() },
            receiver: Receiver { verbosity: Verbosity::HIGH, path: dst.clone() },
            block_size,
            src,
            dst,
        }
    }

    pub fn run(&self, network_client: Option<NetworkClient>) -> bool {
        match network_client {
            None => {
                self.run_local()
            }
            Some(client) => {
                self.run_remote(client)
            }
        }
    }

    fn run_remote(&self, network_client: NetworkClient) -> bool {
        debug!(target: "Client.run_remote", "Remote: {:?} -> {:?}", self.src, self.dst);
        true
    }

    fn run_local(&self) -> bool {
        debug!(target: "Client.run_local", "Local: {:?} -> {:?}", self.src, self.dst);
        if self.src.is_dir() {
            let path_iterator = RecursiveDirectoryIterator::new(&self.src).unwrap();
            for file_result in path_iterator {
                match file_result {
                    Ok(file) => self.run_on_file(&file),
                    Err(e) => eprintln!("Error: {}", e)
                }
            }
        } else {
            self.run_on_file(&self.src);
        }
        true
    }

    fn run_on_file(&self, file: &PathBuf) {
        let suffix = get_suffix(file.as_path(), &self.src).unwrap();
        let file_info = self.sender.get_file_chunks(file).unwrap();
        debug!(target: "Client.run_local", "Read {} chunk data from {:?}", file_info.data.len(), file_info.path_suffix);
        match self.receiver.compare(suffix.as_path(), &file_info, self.block_size) {
            Some(diff_data) => {
                debug!(target: "Client.run_local", "matches {} out of {} chunks", diff_data.common.len(), file_info.data.len());
            }
            None => debug!(target: "Client.run_local", "File does not exist at destination!")
        }
    }
}