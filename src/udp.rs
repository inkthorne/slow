use std::net::UdpSocket;
use serde_json::Value;

pub fn listen_for_udp_packets(port: u16) {
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
                        println!("Received {} bytes from {}: {:?}", amt, src, json);
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
