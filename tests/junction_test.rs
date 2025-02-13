use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use serde_json::json;
use slow::junction::SlowJunction;
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receive_addressed_packet_line() {
        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1110);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 2220);
        let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 3330);
        let addr4 = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4440);

        let junction1 = SlowJunction::new(addr1, 1).expect("Failed to create junction1");
        let junction2 = SlowJunction::new(addr2, 2).expect("Failed to create junction2");
        let junction3 = SlowJunction::new(addr3, 3).expect("Failed to create junction3");
        let junction4 = SlowJunction::new(addr4, 4).expect("Failed to create junction4");

        junction1.seed(addr2);
        junction2.seed(addr3);
        junction3.seed(addr4);

        let ping = json!({"key": "ping"});
        junction1.send(ping.clone(), 4);

        // Delay before receiving the packet
        thread::sleep(Duration::from_millis(1000));

        // Check waiting packet count before receiving
        assert_eq!(junction4.waiting_packet_count(), 1);

        // Check if the datagram was received by junction4
        let received_packet = junction4.recv().unwrap();
        assert_eq!(received_packet.json, ping);
        assert_eq!(received_packet.addr, addr3);

        // Send pong response back to junction1
        let pong = json!({"key": "pong"});
        junction4.send(pong.clone(), 1);

        // Wait for pong to arrive
        thread::sleep(Duration::from_millis(1000));

        // Check waiting packet count before receiving pong
        assert_eq!(junction1.waiting_packet_count(), 1);

        // Verify junction1 received the pong
        let pong_packet = junction1.recv().unwrap();
        assert_eq!(pong_packet.json, pong);
        assert_eq!(pong_packet.addr, addr2);
    }
}