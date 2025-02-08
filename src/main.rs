use std::thread;
use serde_json::json;
mod udp;

fn main() {
    println!("Hello, world!");
    thread::spawn(|| {
        udp::listen_for_udp_packets(2345);
    });

    thread::spawn(|| {
        let json_message_with_slow = json!({
            "message": "Hello from sender!",
            "slow": "1.0"
        }).to_string();
        udp::send_udp_packet(&json_message_with_slow);

        let json_message_without_slow = json!({
            "message": "Hello from sender!"
        }).to_string();
        udp::send_udp_packet(&json_message_without_slow);
    });

    // Keep the main thread alive to allow other threads to run
    loop {}
}
