use crate::json_connection::JsonConnection;
use rand::Rng;
use serde_json::Value;
use std::net::SocketAddr;

pub fn send_json_packet(json_message: &str, addr: SocketAddr) -> std::io::Result<()> {
    let mut rng = rand::thread_rng();
    let bind_port: u16 = rng.gen_range(1024..65535);
    let connection = JsonConnection::new(bind_port)?;
    let json: Value = serde_json::from_str(json_message)?;
    connection.send(&addr.to_string(), &json)
}
