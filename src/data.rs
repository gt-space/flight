use std::net::{SocketAddr, UdpSocket, Ipv4Addr, IpAddr};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::state::State;

// let sys_time = SystemTime::now();

pub struct DataReceiver {
    data_socket: UdpSocket,
    time: SystemTime,
    state: Arc<RwLock<State>>
}

impl DataReceiver {
    pub fn new(state: Arc<RwLock<State>>) -> DataReceiver {
        let data_socket =
            UdpSocket::bind("0.0.0.0:4573").expect("Couldn't bind data_socket to address");
        data_socket
            .set_nonblocking(false)
            .expect("Couldn't set data socket to be non-blocking");
        let time = SystemTime::now();
        DataReceiver { data_socket, time, state }
    }

    pub fn receive(&mut self) -> Result<(usize, SocketAddr), std::io::Error> {
        let mut buf = [0; 1024];
        let (amt, src) = self.data_socket.recv_from(&mut buf)?;
        // println!("Received {} bytes from {} with delay {}", amt, src, self.time.elapsed().unwrap().as_millis());
        self.time = SystemTime::now();

        let socket = UdpSocket::bind("0.0.0.0:7202").expect("couldn't bind to address");
        socket.connect("192.168.0.165:7201").expect("connect function failed");
        socket.send(&buf).expect("couldn't send message");
        Ok((amt, src))
    }
}
