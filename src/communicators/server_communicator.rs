use std::net::{SocketAddr, UdpSocket, TcpStream, IpAddr, Ipv4Addr};
use fs_protobuf_rust::compiled::mcfs::device::DeviceType;
use std::io::prelude::*;
use std::collections::HashMap;
use crate::communicators::Communicator;

// The Control Server Communicator runs on the FS Flight Computer
// Uses TCP to communicate with the Control Server and UDP to the Board Communicator 
pub struct ControlServerCommunicator {
    addr: SocketAddr,
    socket: Option<UdpSocket>,
    server: Option<TcpStream>,
    mappings: HashMap<u32, (DeviceType, SocketAddr)>,
    deployed: bool,
}

pub fn begin(server_comm: &mut ControlServerCommunicator) {
    // let server_addr = server_comm.get_mappings(&150);

    // if let Some((_, address)) = server_addr {
    //     server_comm.server_connect(&address);
    // }
    
    // PROTOBUF MESSAGE STARTS HERE 
    // INSERT MESSAGE TO SEND FROM FC to SERVER 
    // PROTOBUF MESSAGE ENDS HERE 

    loop {
        // if let Some((_, address)) = destination {
        //     println!("address: {:?}", address.to_string());
        //     let sent_bytes = board_comm.send(&data_serialized, address);
        //     println!("bytes sent: {:?}", sent_bytes);
        // }
    }
}

impl Communicator for ControlServerCommunicator {
    fn get_mappings(&self, board_id: &u32) -> Option<(DeviceType, &SocketAddr)> {
        if let Some((dev_type, address)) = self.mappings.get(board_id) {
            Some((*dev_type, address))
        } else {
            panic!("Couldn't access mapping")
        }
    }

    fn update_mappings(&mut self, new_hashmap: HashMap<u32, (DeviceType, SocketAddr)>) -> HashMap<u32, (DeviceType, SocketAddr)> {
        println!("inside update mappings");

        for (key, value) in new_hashmap.iter() {
            self.mappings.insert(*key, *value);
        }
        
        self.mappings.clone()
    }
}

impl ControlServerCommunicator {
    // Constructs a new instance of ['ControlServerCommunicator']
    pub fn new(addr: SocketAddr) -> ControlServerCommunicator {
        ControlServerCommunicator {
            addr, 
            socket: None,
            server: None,
            mappings: HashMap::new(),
            deployed: false,
        }
    }

    // Connected to the Control Server via TCP
    pub fn server_connect(&mut self, server_addr: &SocketAddr) {
        if let Ok(server) = TcpStream::connect(server_addr) {
            self.server = Some(server);
        } else {
            panic!("Failed to connect");
        }
    }

    // Attaches a UDP socket to the provided IP address and port 
    pub fn send_bind(&mut self) {
        if let Ok(socket) = UdpSocket::bind(self.addr) {
            self.socket = Some(socket);
            self.deployed = true;
        } else {
            panic!("Could not attach socket to address and port");
        }
    }

    // Sends data to the Control Server over TCP
    pub fn send_server(&mut self, message: &Vec<u8>) -> usize {
        if let Some(ref mut server) = self.server {
            let sent_bytes = server.write(message).expect("Failed to send message");
            println!("{:?} bytes sent from {:?}", sent_bytes, self.addr);
            return sent_bytes;
        }

        panic!("The stream hasn't been initialized yet");
    }

    // Sends data to the Board Communicator over UDP
    pub fn send_udp(&self, message: &Vec<u8>, dst: &SocketAddr) -> usize {
        if let Some(ref socket) = self.socket {
            let sent_bytes = socket.send_to(message, &dst).expect("failed to send message");
            println!("{:?} bytes sent from {:?}", sent_bytes, self.addr);
            return sent_bytes;
        } 
        
        panic!("The socket hasn't been initialized yet");
    }

    // Reads in data from the control server over TCP
    // Forwards data to FC or Board Communicator 
    pub fn listen_server(&mut self, buf: &mut Vec<u8>) -> usize {
        if let Some(ref mut stream) = self.server {
            let num_bytes = stream.read(buf).expect("Failed to receive data from control server");
            println!("{:?} bytes received", num_bytes);

            let (_, _, routing_addr) = self.parse(&buf);

            if let Some(addr) = routing_addr {
                self.send_udp(buf, addr);
            }

            return num_bytes;
        } 
        panic!("The server stream hasn't been initialized yet");
    }   

    // Reads in data over UDP 
    pub fn listen_board(&mut self, buf: &mut Vec<u8>) -> (usize, SocketAddr) {
        if let Some(ref socket) = self.socket {
            let (num_bytes, src_addr) = socket.recv_from(buf).expect("Failed to receive data");
            println!("{:?} bytes received from {:?}", num_bytes, src_addr);

            let (_board_id, _, routing_addr) = self.parse(&buf);

            if let Some(addr) = routing_addr {
                if addr.ip() == IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)) {
                    // send to FC (self)
                    self.send_udp(buf, addr);
                } else {
                    self.send_server(buf);
                }
            }

            return (num_bytes, src_addr);
        } 
        panic!("The socket hasn't been initialized yet");
    }
}