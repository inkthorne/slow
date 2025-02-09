use rand::Rng;
use serde_json::json;
use serde_json::Value;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;
mod connection;
mod json_connection;

fn main() {
    println!("Hello, world!");
    let mut connection =
        connection::SlowConnection::new(2345).expect("Couldn't create SlowConnection"); // Handle the Result
    thread::spawn(move || {
        connection.listen(on_packet_received);
    });

    thread::spawn(|| {
        let addr: SocketAddr = "[::1]:2345".parse().expect("Invalid address");
        loop {
            let mut rng = rand::thread_rng();
            let bind_port: u16 = rng.gen_range(1024..65535); // Generate a random port number
            let connection = json_connection::JsonConnection::new(bind_port)
                .expect("Couldn't create JsonConnection");
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
