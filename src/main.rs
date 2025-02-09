use serde_json::json;
use serde_json::Value;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
mod connection;
mod json_connection;
mod udp;

fn main() {
    println!("Hello, world!");
    let mut connection = connection::SlowConnection::new();
    thread::spawn(move || {
        connection.listen(2345, on_packet_received);
    });

    thread::spawn(|| {
        let addr: SocketAddr = "[::1]:2345".parse().expect("Invalid address");
        loop {
            let json_message = json!({
                "message": "Hello from sender!",
                "slow": "0.1"
            })
            .to_string();
            udp::send_json_packet(&json_message, addr).expect("Failed to send JSON packet");
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Keep the main thread alive to allow other threads to run
    loop {}
}

fn on_packet_received(json: &Value) {
    println!("Callback received JSON: {:?}", json);
}
