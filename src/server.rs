use std::net::TcpListener;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::io::Read;
use log::{info};

pub trait ServerTrait {
    fn run(&self);
}

pub struct TcpServer {
    address: String
}

impl ServerTrait for TcpServer {
    fn run(&self) {
        let listener = TcpListener::bind(&self.address).unwrap();
        info!(target: "TcpServer", "TCP Server is running: {:?}", self.address);
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            self.handle_connection(stream);
        }
    }
}

impl TcpServer {
    pub fn new(address: &str) -> Self {
        TcpServer { address: address.to_string() }
    }

    pub fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        info!(target: "TcpServer", "Received: {:?}", buffer);
    }
}

pub struct UdpServer {
    address: String
}

impl ServerTrait for UdpServer {
    fn run(&self) {
        let socket = UdpSocket::bind(&self.address).unwrap();
        info!(target: "UdpServer", "UDP Server running on {}", &self.address);

        let mut buf = [0; 1024];
        loop {
            let (size, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
            info!(target: "UdpServer", "Received data from {}: {:?}", src_addr, size);
        }
    }
}

impl UdpServer {
    pub fn new(address: &str) -> Self {
        UdpServer {
            address: address.to_string(),
        }
    }
}

enum Server {
    UDP(UdpServer),
    TCP(TcpServer),
}

// Implement the ServerTrait for the Server enum
impl ServerTrait for Server {
    fn run(&self) {
        match self {
            Server::UDP(udp_server) => udp_server.run(),
            Server::TCP(tcp_server) => tcp_server.run(),
        }
    }
}