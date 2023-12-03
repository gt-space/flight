use std::{time::Duration, thread, net::UdpSocket};
use tracing::info;

use crate::{discovery::get_ips, state};

pub struct ForwardingAgent {
    frequency: Duration
}

impl ForwardingAgent {
    pub fn new() -> ForwardingAgent {
        ForwardingAgent {
            frequency: Duration::from_millis(100)
        }
    }

    pub fn begin_forward(&mut self) {
        loop {
            if let Some(server_ip) = get_ips(&["server-01.local"]).get("server-01.local") {
                info!("Found server at {}", server_ip);
                let socket = UdpSocket::bind("0.0.0.0:8765").expect("Couldn't bind socket to address");
                socket.connect(format!("{}:7201", server_ip)).expect("Couldn't connect to server");
                self.forward_loop(socket);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn forward_loop(&mut self, socket: UdpSocket) {
        loop {
            let vehciel_state = state::get_vehicle_state();
            let buffer = postcard::to_allocvec(&vehciel_state);
            if let Ok(buffer) = buffer {
                if let Ok(_size) = socket.send(&buffer) {
                    info!("Sent {} bytes", buffer.len());
                }
            }
            thread::sleep(self.frequency);
        }
    }
}