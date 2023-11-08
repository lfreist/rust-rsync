use std::any::{Any, TypeId};
use std::fs::DirEntry;
use std::{fs, io};
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::net::UdpSocket;
use std::io::Write;
use log::{info, debug, error};
use std::path::{Path, PathBuf};
use std::vec::IntoIter;

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
    HIGH
}

pub struct RsyncClient {
    verbosity: Verbosity
}

impl RsyncClient {
    pub fn new(verbosity: Verbosity) ->RsyncClient {
        RsyncClient { verbosity }
    }

    pub fn run(&self, src: &Path, dst: &Path, network_client: Option<NetworkClient>) -> bool {
        match network_client {
            None => {
                self.run_local(src, dst)
            }
            Some(client) => {
                self.run_remote(src, dst, client)
            }
        }
    }

    fn run_remote(&self, src: &Path, dst: &Path, network_client: NetworkClient) -> bool {
        debug!(target: "RsyncClient", "Remote: {:?} -> {:?}", src, dst);
        true
    }

    fn run_local(&self, src: &Path, dst: &Path) -> bool {
        debug!(target: "RsyncClient", "Local: {:?} -> {:?}", src, dst);
        if src.is_dir() {

        }
        true
    }
}