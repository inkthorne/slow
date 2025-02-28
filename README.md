# Slow

Slow is a decentralized network communication library written in Rust, designed for resilient, peer-to-peer package routing with built-in mesh networking capabilities.

## Overview

Slow provides a framework for creating and managing network junctions that can communicate with each other through a mesh topology. The library supports automatic route discovery, package forwarding, and reliable message delivery across distributed nodes.

## Key Features

- **Mesh Network Topology**: Nodes (junctions) can discover and communicate with each other without central coordination
- **Automatic Route Discovery**: Optimizes message delivery paths through the network
- **Multiple Package Types**: Supports various types of data payloads (JSON, Binary, Control messages)
- **Hop-based Forwarding**: Limits package propagation through configurable hop counts
- **Resilient Communication**: Designed to maintain connectivity in dynamic network conditions

## Core Components

### Junction

The central networking entity that:
- Manages connections with other junctions
- Routes packages through the network
- Maintains a routing table for optimized delivery
- Processes incoming and outgoing messages

### Package

Data container with:
- Headers for routing information
- Support for JSON and binary payloads
- Built-in hop counting to prevent infinite loops

### Socket

Low-level component that:
- Handles network I/O operations
- Serializes/deserializes packages for transmission

### Route Table

Maintains efficient routing information:
- Tracks best paths to other junctions
- Updates dynamically based on network conditions

## Usage Examples

Basic usage involves creating junctions, connecting them to form a network, and sending messages between them:

```rust
// Create a new junction
let junction = SlowJunction::new(socket_addr, junction_id).await?;

// Join an existing network by connecting to a known node
junction.join(seed_addr).await;

// Send a JSON message to another junction
junction.send(json_value, &recipient_id).await;

// Receive messages
if let Some(packet) = junction.recv().await {
    // Process received JSON packet
}
```

## Building and Testing

This project uses Cargo for building and testing:

```
cargo build
cargo test
```

## License

[Add license information]
