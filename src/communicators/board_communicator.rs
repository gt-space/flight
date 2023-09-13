use std::net::{SocketAddr, UdpSocket};
use std::collections::HashMap;
use crate::communicators::Communicator;
use fs_protobuf_rust::compiled::mcfs::device::DeviceType;

// Board Communicator runs on the FS Flight Computer
// Relays messages to and from the Control Server Communicator, the SAM modules, and the BMS
// Uses the User Datagram Protocol (UDP)
pub struct BoardCommunicator {
    addr: SocketAddr,
    socket: Option<UdpSocket>,
    mappings: HashMap<u32, (DeviceType, SocketAddr)>,
    deployed: bool,
}

pub fn begin(board_comm: &mut BoardCommunicator, forwarded_message: Vec<u8>) {
    board_comm.send_bind();

    let (_, _, routing_addr) = board_comm.parse(&forwarded_message);

    if let Some(addr) = routing_addr {
        // send data to destination  
        let sent_bytes = board_comm.send(&forwarded_message, addr);
        println!("bytes sent: {:?}", sent_bytes);
    }

    // ADDRESS BELOW FOR COMMAND LOOP SOCKET ON SAM 
    //let sam_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(169, 254, 42, 143)), 8378);
}

impl Communicator for BoardCommunicator {
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

impl BoardCommunicator {
    // Constructs a new instance of ['BoardCommunicator']
    pub fn new(addr: SocketAddr) -> BoardCommunicator {
        BoardCommunicator {
            addr,
            socket: None, 
            mappings: HashMap::new(),
            deployed: false,
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

    pub fn send(&self, message: &Vec<u8>, dst: &SocketAddr) -> usize {
        if let Some(ref socket) = self.socket {
            println!("message: {:?}, dst: {:?}", message, dst.to_string().as_str());
            let sent_bytes = socket.send_to(message, dst.to_string()).expect("failed to send message");
            println!("{:?} bytes sent from {:?}", sent_bytes, self.addr);
            return sent_bytes;
        } 
        
        panic!("The socket hasn't been initialized yet");
    }

    // Reads in data over UDP
    pub fn listen(&mut self, buf: &mut Vec<u8>) -> (usize, SocketAddr) {
        if let Some(ref socket) = self.socket {
            let (num_bytes, src_addr) = socket.recv_from(buf).expect("Failed to receive data");
            println!("{:?} bytes received from {:?}", num_bytes, src_addr);

            let (_, _, routing_addr) = self.parse(&buf);

            if let Some(addr) = routing_addr {
                self.send(buf, addr);
            }

            return (num_bytes, src_addr);
        } 

        panic!("The socket hasn't been initialized yet");
    }
}

