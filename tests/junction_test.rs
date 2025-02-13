use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use serde_json::json;
use slow::junction::SlowJunction;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receive_addressed_packet() {
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1110);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2220);
        let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3330);

        let junction1 = SlowJunction::new(addr1, 1).expect("Failed to create junction1");
        let junction2 = SlowJunction::new(addr2, 2).expect("Failed to create junction2");
        let junction3 = SlowJunction::new(addr3, 3).expect("Failed to create junction3");

        junction1.seed(addr2);
        junction2.seed(addr3);

        let ping = json!({"key": "ping"});
        junction1.send(ping.clone(), 3);

        // Delay before receiving the packet
        thread::sleep(Duration::from_millis(1000));

        // Check if the datagram was received by junction3
        let received_packet = junction3.recv().unwrap();
        assert_eq!(received_packet.json, ping);
        assert_eq!(received_packet.addr, addr2);

        // Send pong response back to junction1
        let pong = json!({"key": "pong"});
        junction3.send(pong.clone(), 1);

        // Wait for pong to arrive
        thread::sleep(Duration::from_millis(1000));

        // Verify junction1 received the pong
        let pong_packet = junction1.recv().unwrap();
        assert_eq!(pong_packet.json, pong);
        assert_eq!(pong_packet.addr, addr2);
    }
}