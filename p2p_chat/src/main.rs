// P2P Chat Application
// Author: JIN SEUNG EON
// Student ID: 20201406

use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time::interval; 

// XXXX with last 4 digits of student ID
const XXXX: u16 = 1406;

// host vars
const HOST_LOCAL: &str = "127.0.0.1";
// const HOST_A: &str = "nsl5.cau.ac.kr";
// const HOST_B: &str = "nsl2.cau.ac.kr";

// Hardcoded peer addresses (XXXX with student ID digits)
const PEER_ADDRESSES: [(u8, &str, u16); 4] = [
    (1, HOST_LOCAL, 2*10000 + XXXX),
    (2, HOST_LOCAL, 3*10000 + XXXX),
    (3, HOST_LOCAL, 4*10000 + XXXX),
    (4, HOST_LOCAL, 5*10000 + XXXX),
];

const K: usize = 2; // Maximum number of connections per direction
const RECONNECT_INTERVAL: Duration = Duration::from_secs(3);
const CONNECT_MORE_INTERVAL: Duration = Duration::from_secs(10);

// Message types for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Message {
    ConnectionRequest {
        from_node: u8,
        nickname: String,
    },
    ConnectionAck {
        from_node: u8,
        nickname: String,
    },
    ConnectionFail {
        from_node: u8,
        reason: String,
    },
    ConnectionClosed {
        source_node: u8, 
        sequence_number: u32,
        from_node: u8,
        nickname: String,
    },
    ChatMessage {
        source_node: u8,
        sequence_number: u32,
        from_node: u8,
        nickname: String,
        content: String,
    },
}

// Information about connected peers
#[derive(Debug, Clone)]
struct PeerInfo {
    address: SocketAddr,
    nickname: Option<String>,
    is_outgoing: bool,
    last_seen: Instant,
}

// Unique identifier for chat messages to prevent duplicates
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MessageId {
    source_node: u8,
    sequence_number: u32,
}

// Main P2P node structure
struct P2PNode {
    node_id: u8,
    nickname: String,
    socket: Arc<UdpSocket>,
    peers: Arc<RwLock<HashMap<u8, PeerInfo>>>,
    message_cache: Arc<RwLock<HashSet<MessageId>>>,
    sequence_number: Arc<Mutex<u32>>,
    shutdown_tx: broadcast::Sender<()>,
}

impl P2PNode {
    async fn new(node_id: u8, nickname: String) -> Result<Self, Box<dyn std::error::Error>> {
        let bind_addr = PEER_ADDRESSES.iter()
            .find(|(id, _, _)| *id == node_id)
            .ok_or("Invalid node ID")?;
        
        let socket = UdpSocket::bind(("0.0.0.0", bind_addr.2)).await?;
        println!("Node {} ({}) listening on port {}", node_id, nickname, bind_addr.2);

        let (shutdown_tx, _) = broadcast::channel(1);

        Ok(P2PNode {
            node_id,
            nickname,
            socket: Arc::new(socket),
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_cache: Arc::new(RwLock::new(HashSet::new())),
            sequence_number: Arc::new(Mutex::new(0)),
            shutdown_tx,
        })
    }

    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        
        // Start background tasks
        let socket_clone = Arc::clone(&self.socket);
        let peers_clone = Arc::clone(&self.peers);
        let message_cache_clone = Arc::clone(&self.message_cache);
        let nickname_clone = self.nickname.clone();
        let node_id = self.node_id;

        // Message receiver task
        let mut shutdown_rx_receiver = self.shutdown_tx.subscribe();
        let receiver_task = tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            loop {
                tokio::select! {
                    _ = shutdown_rx_receiver.recv() => break,
                    result = socket_clone.recv_from(&mut buf) => {
                        match result {
                            Ok((len, addr)) => {
                                if let Ok(msg) = serde_json::from_slice::<Message>(&buf[..len]) {
                                    Self::handle_message(
                                        msg,
                                        addr,
                                        node_id,
                                        &nickname_clone,
                                        &peers_clone,
                                        &message_cache_clone,
                                        &socket_clone,
                                    ).await;
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::ConnectionReset => {
                                // Ignore connection reset errors
                                continue;
                            }
                            Err(e) => eprintln!("Error receiving message: {}", e),
                        }
                    }
                }
            }
        });

        // Connection management task
        let peers_clone = Arc::clone(&self.peers);
        let socket_clone = Arc::clone(&self.socket);
        let nickname_clone = self.nickname.clone();
        let node_id = self.node_id;
        let mut shutdown_rx2 = self.shutdown_tx.subscribe();

        let connection_task = tokio::spawn(async move {
            let mut reconnect_timer = interval(RECONNECT_INTERVAL);
            let mut connect_more_timer = interval(CONNECT_MORE_INTERVAL);

            loop {
                tokio::select! {
                    _ = shutdown_rx2.recv() => break,
                    _ = reconnect_timer.tick() => {
                        let peer_count = peers_clone.read().await.len();
                        if peer_count == 0 {
                            Self::try_establish_connections(
                                node_id,
                                &nickname_clone,
                                &peers_clone,
                                &socket_clone,
                                true,
                            ).await;
                        }
                    }
                    _ = connect_more_timer.tick() => {
                        let peer_count = peers_clone.read().await.len();
                        if peer_count < K {
                            Self::try_establish_connections(
                                node_id,
                                &nickname_clone,
                                &peers_clone,
                                &socket_clone,
                                false,
                            ).await;
                        }
                    }
                }
            }
        });

        // Initial connection attempt
        Self::try_establish_connections(self.node_id, &self.nickname, &self.peers, &self.socket, true).await;

        // User input handler
        let socket_clone = Arc::clone(&self.socket);
        let peers_clone = Arc::clone(&self.peers);
        let sequence_number_clone = Arc::clone(&self.sequence_number);
        let nickname_clone = self.nickname.clone();
        let node_id = self.node_id;
        let shutdown_tx_clone = self.shutdown_tx.clone();

        let input_task = tokio::spawn(async move {
            let mut buf = String::new();

            loop {
                print!("> ");
                io::stdout().flush().unwrap();
                buf.clear();
                
                match std::io::stdin().read_line(&mut buf) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let input = buf.trim();
                        if input.is_empty() {
                            continue;
                        }

                        match input {
                            "\\quit" => {
                                // Send connection closed message and shutdown
                                Self::send_connection_closed_message(
                                    node_id,
                                    &nickname_clone,
                                    &peers_clone,
                                    &socket_clone,
                                    &sequence_number_clone,
                                ).await;
                                let _ = shutdown_tx_clone.send(());
                                break;
                            }
                            "\\list" => {
                                // Show connected peers
                                Self::show_peer_list(&peers_clone).await;
                            }
                            "\\help" => {
                                // Show help message
                                Self::show_help();
                            }
                            _ => {
                                // Broadcast chat message to all peers
                                Self::broadcast_chat_message(
                                    input,
                                    node_id,
                                    &nickname_clone,
                                    &peers_clone,
                                    &socket_clone,
                                    &sequence_number_clone,
                                ).await;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading input: {}", e);
                        break;
                    }
                }
            }
        });

        // Wait for shutdown
        tokio::select! {
            _ = receiver_task => {},
            _ = connection_task => {},
            _ = input_task => {},
        }

        Ok(())
    }

    // Try to establish connections with other nodes
    async fn try_establish_connections(
        node_id: u8,
        nickname: &str,
        peers: &Arc<RwLock<HashMap<u8, PeerInfo>>>,
        socket: &Arc<UdpSocket>,
        is_urgent: bool,
    ) {
        let current_peers = peers.read().await;
        let outgoing_count = current_peers.values().filter(|p| p.is_outgoing).count();
        let total_count = current_peers.len();
        drop(current_peers);

        if outgoing_count >= K || total_count >= K + 1 {
            return;
        }

        let mut available_peers: Vec<_> = PEER_ADDRESSES.iter()
            .filter(|(id, _, _)| *id != node_id)
            .collect();

        available_peers.shuffle(&mut rand::thread_rng());

        for (peer_node_id, host, port) in available_peers {
            let current_peers = peers.read().await;
            let outgoing_count = current_peers.values().filter(|p| p.is_outgoing).count();
            let total_count = current_peers.len();
            
            if current_peers.contains_key(peer_node_id) {
                continue; // Skip if already connected
            }
            drop(current_peers);

            if outgoing_count >= K || total_count >= K + 1 {
                break;
            }

            // Resolve DNS to IP address
            let addr_str = format!("{}:{}", host, port);
            match addr_str.to_socket_addrs() {
                Ok(mut addrs) => {
                    if let Some(resolved_addr) = addrs.next() {
                        let request = Message::ConnectionRequest {
                            from_node: node_id,
                            nickname: nickname.to_string(),
                        };

                        if let Ok(data) = serde_json::to_vec(&request) {
                            let _ = socket.send_to(&data, resolved_addr).await;
                            
                            if is_urgent {
                                tokio::time::sleep(Duration::from_millis(500)).await;
                            }
                        }
                    } else {
                        eprintln!("No address found for {}", addr_str);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to resolve {}: {}", addr_str, e);
                }
            }
        }
    }

    // Handle incoming messages
    async fn handle_message(
        message: Message,
        sender_addr: SocketAddr,
        node_id: u8,
        nickname: &str,
        peers: &Arc<RwLock<HashMap<u8, PeerInfo>>>,
        message_cache: &Arc<RwLock<HashSet<MessageId>>>,
        socket: &Arc<UdpSocket>,
    ) {
        match message {
            Message::ConnectionRequest { from_node, nickname: peer_nickname } => {
                let mut peers_lock = peers.write().await;
                let incoming_count = peers_lock.values().filter(|p| !p.is_outgoing).count();
                let total_count = peers_lock.len();

                if peers_lock.contains_key(&from_node) {
                    // Already connected
                    let response = Message::ConnectionFail {
                        from_node: node_id,
                        reason: "Already connected".to_string(),
                    };
                    if let Ok(data) = serde_json::to_vec(&response) {
                        let _ = socket.send_to(&data, sender_addr).await;
                    }
                } else if incoming_count >= K || total_count >= K + 1 {
                    // Connection limit reached
                    let response = Message::ConnectionFail {
                        from_node: node_id,
                        reason: "Connection limit reached".to_string(),
                    };
                    if let Ok(data) = serde_json::to_vec(&response) {
                        let _ = socket.send_to(&data, sender_addr).await;
                    }
                } else {
                    // Accept connection
                    peers_lock.insert(from_node, PeerInfo {
                        address: sender_addr,
                        nickname: Some(peer_nickname),
                        is_outgoing: false,
                        last_seen: Instant::now(),
                    });

                    let response = Message::ConnectionAck {
                        from_node: node_id,
                        nickname: nickname.to_string(),
                    };
                    if let Ok(data) = serde_json::to_vec(&response) {
                        let _ = socket.send_to(&data, sender_addr).await;
                    }
                    // Print when connection is established (incoming)
                    println!("Peer {} connected", from_node);
                }
            }

            Message::ConnectionAck { from_node, nickname: peer_nickname } => {
                let mut peers_lock = peers.write().await;
                // Print message only when registering from outgoing connection side
                if let Some(peer) = peers_lock.get_mut(&from_node) {
                    // Do not print if already registered (reconnection, etc.)
                    peer.nickname = Some(peer_nickname);
                    peer.last_seen = Instant::now();
                } else {
                    // Print only when first registered
                    peers_lock.insert(from_node, PeerInfo {
                        address: sender_addr,
                        nickname: Some(peer_nickname),
                        is_outgoing: true,
                        last_seen: Instant::now(),
                    });
                    println!("Peer {} connected", from_node);
                }
            }

            Message::ConnectionFail { from_node: _, reason: _ } => {
                // Connection failed, nothing to do
            }

           Message::ConnectionClosed { source_node, sequence_number, from_node: _, nickname: peer_nickname } => {
            let message_id = MessageId { source_node, sequence_number };
            
            let mut cache = message_cache.write().await;
            if cache.contains(&message_id) {
                return; // Already processed this message
            }
            cache.insert(message_id);
            drop(cache);

            // Remove peer and display message
            let mut peers_lock = peers.write().await;
            if peers_lock.remove(&source_node).is_some() {
                println!("{} has left the chat", peer_nickname);
            }
            drop(peers_lock);

            // Forward to other peers (except the one it came from)
            let peers_lock = peers.read().await;
            for (_, peer_info) in peers_lock.iter() {
                let forward_msg = Message::ConnectionClosed {
                    source_node,
                    sequence_number,
                    from_node: node_id,
                    nickname: peer_nickname.clone(),
                };
                
                if let Ok(data) = serde_json::to_vec(&forward_msg) {
                    let _ = socket.send_to(&data, peer_info.address).await;
                }
            }
        }

            Message::ChatMessage { source_node, sequence_number, from_node, nickname: sender_nickname, content } => {
                let message_id = MessageId { source_node, sequence_number };
                
                let mut cache = message_cache.write().await;
                if cache.contains(&message_id) {
                    return; // Already processed this message
                }
                cache.insert(message_id);
                drop(cache);

                // Display the message
                println!("{}> {}", sender_nickname, content);

                // Forward to other peers (except the one it came from)
                let peers_lock = peers.read().await;
                for (peer_id, peer_info) in peers_lock.iter() {
                    if *peer_id != from_node {
                        let forward_msg = Message::ChatMessage {
                            source_node,
                            sequence_number,
                            from_node: node_id,
                            nickname: sender_nickname.clone(),
                            content: content.clone(),
                        };
                        
                        if let Ok(data) = serde_json::to_vec(&forward_msg) {
                            let _ = socket.send_to(&data, peer_info.address).await;
                        }
                    }
                }
            }
        }
    }

    // Broadcast chat message to all connected peers
    async fn broadcast_chat_message(
        content: &str,
        node_id: u8,
        nickname: &str,
        peers: &Arc<RwLock<HashMap<u8, PeerInfo>>>,
        socket: &Arc<UdpSocket>,
        sequence_number: &Arc<Mutex<u32>>,
    ) {
        let mut seq_num = sequence_number.lock().await;
        *seq_num += 1;
        let current_seq = *seq_num;
        drop(seq_num);

        let message = Message::ChatMessage {
            source_node: node_id,
            sequence_number: current_seq,
            from_node: node_id,
            nickname: nickname.to_string(),
            content: content.to_string(),
        };

        if let Ok(data) = serde_json::to_vec(&message) {
            let peers_lock = peers.read().await;
            for peer_info in peers_lock.values() {
                let _ = socket.send_to(&data, peer_info.address).await;
            }
        }
    }

    // Send connection closed message to all peers when shutting down
    async fn send_connection_closed_message(
        node_id: u8,
        nickname: &str,
        peers: &Arc<RwLock<HashMap<u8, PeerInfo>>>,
        socket: &Arc<UdpSocket>,
        sequence_number: &Arc<Mutex<u32>>,
    ) {
        let mut seq_num = sequence_number.lock().await;
        *seq_num += 1;
        let current_seq = *seq_num;
        drop(seq_num);

        let message = Message::ConnectionClosed {
            source_node: node_id,
            sequence_number: current_seq,
            from_node: node_id,
            nickname: nickname.to_string(),
        };

        if let Ok(data) = serde_json::to_vec(&message) {
            let peers_lock = peers.read().await;
            for peer_info in peers_lock.values() {
                let _ = socket.send_to(&data, peer_info.address).await;
            }
        }
    }

    // Display list of connected peers
    async fn show_peer_list(peers: &Arc<RwLock<HashMap<u8, PeerInfo>>>) {
        let peers_lock = peers.read().await;
        println!("Connected peers:");
        for (node_id, peer_info) in peers_lock.iter() {
            let direction = if peer_info.is_outgoing { "outgoing" } else { "incoming" };
            let unknown_nickname = "unknown".to_string();
            let nickname = peer_info.nickname.as_ref().unwrap_or(&unknown_nickname);
            println!("  Node {}: {} ({}) - {}", node_id, peer_info.address, direction, nickname);
        }
        println!("Total connections: {}", peers_lock.len());
    }

    // Display help message
    fn show_help() {
        println!("Available commands:");
        println!("  \\list    - Show connected peers");
        println!("  \\help    - Show this help message");
        println!("  \\quit    - Leave the chat and quit");
        println!("  <message> - Send a chat message to all peers");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <node_id> <nickname>", args[0]);
        eprintln!("Node ID must be 1, 2, 3, or 4");
        std::process::exit(1);
    }

    let node_id: u8 = args[1].parse().map_err(|_| "Invalid node ID")?;
    if !(1..=4).contains(&node_id) {
        eprintln!("Node ID must be 1, 2, 3, or 4");
        std::process::exit(1);
    }

    let nickname = args[2].clone();
    if nickname.len() > 16 || nickname.contains(' ') || nickname.contains('\\') {
        eprintln!("Nickname must be ≤16 chars, no spaces or backslashes");
        std::process::exit(1);
    }

    println!("Starting P2P chat node {} with nickname '{}'", node_id, nickname);
    println!("Type \\help for available commands");

    let node = P2PNode::new(node_id, nickname).await?;
    node.start().await?;

    println!("Goodbye!");
    Ok(())
}