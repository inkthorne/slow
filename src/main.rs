use rand::Rng;
use serde_json::json;
use serde_json::Value;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
mod connection;
mod junction;

fn main() {
    println!("Hello, world!");
    let addr: SocketAddr = "[::1]:2345".parse().expect("Invalid address");
    let mut junction = junction::SlowJunction::new(addr).expect("Couldn't create SlowJunction");
    thread::spawn(move || {
        junction.run(on_packet_received);
    });

    thread::spawn(|| {
        let addr: SocketAddr = "[::1]:2345".parse().expect("Invalid address");
        loop {
            let mut rng = rand::thread_rng();
            let bind_port: u16 = rng.gen_range(1024..65535); // Generate a random port number
            let bind_addr: SocketAddr = format!("[::1]:{}", bind_port)
                .parse()
                .expect("Invalid address");
            let connection =
                connection::JsonConnection::new(bind_addr).expect("Couldn't create JsonConnection");
            let json_message = json!({
                "message": "Hello from sender!",
                "slow": "0.1"
            });
            connection
                .send(&addr.to_string(), &json_message)
                .expect("Failed to send JSON packet");
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Keep the main thread alive to allow other threads to run
    loop {
        thread::sleep(Duration::from_secs(5)); // Add delay of 5 seconds
    }
}

fn on_packet_received(json: &Value) {
    println!("Callback received JSON: {:?}", json);
}
