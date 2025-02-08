use std::net::UdpSocket;
use serde_json::Value;

/// Listens for UDP packets on the specified port.
/// 
/// # Arguments
/// 
/// * `port` - A u16 that specifies the port number to bind to.
/// * `callback` - A function to be called when a packet is received.
/// 
/// This function does not return a value.
pub fn listen_for_udp_packets<F>(port: u16, callback: F)
where
    F: Fn(&Value) + Send + 'static,
{
    let address = format!("localhost:{}", port);
    let socket = UdpSocket::bind(&address).expect("Couldn't bind to address");
    let mut buf = [0; 1024];
    loop {
        let (amt, src) = socket.recv_from(&mut buf).expect("Didn't receive data");
        let data = &buf[..amt];
        match serde_json::from_slice::<Value>(data) {
            Ok(json) => {
                if let Some(slow) = json.get("slow") {
                    if slow == "1.0" {
                        callback(&json);
                    } else {
                        println!("Rejected packet from {}: {:?}", src, json);
                    }
                } else {
                    println!("Rejected packet from {}: {:?}", src, json);
                }
            }
            Err(_) => {
                println!("Ignored invalid JSON packet from {}", src);
            }
        }
    }
}

pub fn send_udp_packet(json_message: &str) {
    let socket = UdpSocket::bind("localhost:5432").expect("Couldn't bind to address");
    socket.send_to(json_message.as_bytes(), "localhost:2345").expect("Couldn't send data");
}
