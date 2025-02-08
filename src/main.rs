use std::net::UdpSocket;
use std::thread;
use serde_json::Value;

fn main() {
    println!("Hello, world!");
    thread::spawn(|| {
        listen_for_udp_packets();
    });

    thread::spawn(|| {
        let json_message = serde_json::json!({
            "message": "Hello from sender!"
        }).to_string();
        send_udp_packet(&json_message);
    });

    // Keep the main thread alive to allow other threads to run
    loop {}
}

fn listen_for_udp_packets() {
    let socket = UdpSocket::bind("localhost:2345").expect("Couldn't bind to address");
    let mut buf = [0; 1024];
    loop {
        let (amt, src) = socket.recv_from(&mut buf).expect("Didn't receive data");
        let data = &buf[..amt];
        let json: Value = serde_json::from_slice(data).expect("Couldn't parse JSON");
        println!("Received {} bytes from {}: {:?}", amt, src, json);
    }
}

fn send_udp_packet(json_message: &str) {
    let socket = UdpSocket::bind("localhost:5432").expect("Couldn't bind to address");
    socket.send_to(json_message.as_bytes(), "localhost:2345").expect("Couldn't send data");
}
