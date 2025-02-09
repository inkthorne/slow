use std::net::{UdpSocket, SocketAddr};
use serde_json::Value;
use rand::Rng;

pub struct SlowPacket {
    pub addr: SocketAddr,
    pub json: Value,
}

/// Listens for a single UDP packet on the specified port and returns the packet details.
/// 
/// # Arguments
/// 
/// * `socket` - A reference to a `UdpSocket`.
/// 
/// # Returns
/// 
/// * `Option<SlowPacket>` - The packet details if the packet is valid and contains the key 'slow' with value '0.1', otherwise None.
pub fn listen_for_slow_packet(socket: &UdpSocket) -> Option<SlowPacket> {
    let mut buf = [0; 1024];
    loop {
        let (amt, src) = socket.recv_from(&mut buf).expect("Didn't receive data");
        let data = &buf[..amt];
        match serde_json::from_slice::<Value>(data) {
            Ok(json) => {
                if let Some(slow) = json.get("slow") {
                    if slow == "0.1" {
                        println!("Received packet from {}:{}", src.ip(), src.port());
                        return Some(SlowPacket {
                            addr: src,
                            json,
                        });
                    } else {
                        println!("Rejected packet from {}: {:?}", src, json);
                    }
                } else {
                    println!("Rejected packet from {}: {:?}", src, json);
                }
            }
            Err(_) => {
                // Silently ignore invalid JSON
            }
        }
    }
}

pub fn send_udp_packet(json_message: &str, addr: SocketAddr) {
    let mut rng = rand::thread_rng();
    let bind_port: u16 = rng.gen_range(1024..65535);
    let bind_address = format!("localhost:{}", bind_port);
    let socket = UdpSocket::bind(&bind_address).expect("Couldn't bind to address");
    socket.send_to(json_message.as_bytes(), &addr).expect("Couldn't send data");
}
