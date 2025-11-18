//20201406 SEUNG EON JIN

const CMD_LIST: u8   = 0x01;
const CMD_TO: u8     = 0x02;
const CMD_EXCEPT: u8 = 0x03;
const CMD_BAN: u8    = 0x04;
const CMD_PING: u8   = 0x05;


use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{TcpListener, TcpStream},
    sync::{broadcast, Mutex},
};
use std::{collections::HashMap, sync::Arc, time::Duration};
// Shared nickname map type
type SharedMap = Arc<Mutex<HashMap<String, String>>>;
// Mapping between nicknames and addresses
type NickToAddrMap = Arc<Mutex<HashMap<String, String>>>;

const MAX_CLIENTS: usize = 4;

async fn reject_client(socket: TcpStream) {
    println!("rejected");
    let mut writer = BufWriter::new(socket);
    let _ = writer
        .write_all(b"chatting room full. cannot connect\n")
        .await;
    let _ = writer.flush().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("write & wait end");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = "11406";
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await?;
    let (tx, _rx) = broadcast::channel::<String>(100);
    // Map from addr -> nickname
    let nickname_map: SharedMap = Arc::new(Mutex::new(HashMap::new()));
    // Map from nickname -> addr (for commands)
    let nick_to_addr_map: NickToAddrMap = Arc::new(Mutex::new(HashMap::new()));

    println!("Chat server running on port {}...", port);

    // Shutdown signal
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);

    // Ctrl+C handler
    {
        let shutdown_signal_tx = shutdown_tx.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
            let _ = shutdown_signal_tx.send(()).await;
        });
    }

    // Debug print of connected users
    {
        let nick_map = Arc::clone(&nickname_map);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                let map = nick_map.lock().await;
                let nicknames: Vec<_> = map.values().cloned().collect();
                println!(
                    "[Info] Connected users ({}): {:?}",
                    nicknames.len(),
                    nicknames
                );
            }
        });
    }
let mut was_banned = false;
    loop {
        tokio::select! {
            Ok((socket, addr)) = listener.accept() => {
                let tx = tx.clone();
                let mut rx = tx.subscribe();
                let nick_map = Arc::clone(&nickname_map);
                let nick_to_addr = Arc::clone(&nick_to_addr_map);

                if nick_map.lock().await.len() >= MAX_CLIENTS {
                    reject_client(socket).await;
                    continue;
                }

                tokio::spawn(async move {
                    let (reader, mut writer) = socket.into_split();
                    let mut reader = BufReader::new(reader).lines();

                    let nickname: String;
                    loop {
                        writer.write_all(b"Please enter your nickname:\n").await.ok();

                        match reader.next_line().await {
                            Ok(Some(input)) if !input.trim().is_empty() => {
                                let mut map = nick_map.lock().await;
                                if map.values().any(|v| v == input.trim()) {
                                    writer.write_all(b"Nickname already used by another user. cannot connect\n").await.ok();
                                    continue;
                                } else {
                                    nickname = input.trim().to_string();
                                    map.insert(addr.to_string(), nickname.clone());
                                    
                                    // Also add to the nickname -> addr map
                                    let mut nick_addr_map = nick_to_addr.lock().await;
                                    nick_addr_map.insert(nickname.clone(), addr.to_string());
                                    
                                    let current_count = map.len();
                                    println!(
                                        "{} joined from {}. There are {} users in the room.",
                                        nickname, addr, current_count
                                    );
                                    break;
                                }
                            }
                            Ok(None) => {
                                println!("Client disconnected during nickname setup");
                                return;
                            }
                            Err(_) => {
                                println!("Error reading from client during nickname setup");
                                return;
                            }
                            _ => {
                                writer.write_all(b"Invalid nickname.\n").await.ok();
                            }
                        }
                    }

                    // Welcome
                    let user_count = nick_map.lock().await.len();
                    writer
                        .write_all(format!("Welcome {} to CAU net-class chat room at 127.0.0.1:{}.\nThere are {} users in the room\n", nickname, port, user_count).as_bytes())
                        .await
                        .ok();

                    let my_nickname_tag = format!("[{}]", nickname);

                    let addr_clone = addr.to_string();
                    let nick_map_clone = Arc::clone(&nick_map);
                    let heartbeat_task = tokio::spawn(async move {
                        let mut interval = tokio::time::interval(Duration::from_secs(15));
                        loop {
                            interval.tick().await;
                            let map = nick_map_clone.lock().await;
                            if !map.contains_key(&addr_clone) {
                                break;
                            }
                        }
                    });


                    loop {
                        tokio::select! {
                            // Process client's message
                            result = reader.next_line() => {
                                match result {
                                    Ok(Some(line)) => {
                                        // Check if it's a command (first byte is a command code)
                                        if !line.is_empty() {
                                            let first_byte = line.as_bytes()[0];
                                            if is_valid_command(first_byte){
                                            match first_byte {

                                                CMD_LIST => {
                                                    // Handle \list command
                                                    let users_list = {
                                                        let map = nick_map.lock().await;
                                                        let users: Vec<_> = map.values().cloned().collect();
                                                        format!("Connected users ({}): {}", users.len(), users.join(", "))
                                                    };
                                                    writer.write_all(users_list.as_bytes()).await.ok();
                                                    writer.write_all(b"\n").await.ok();
                                                    writer.flush().await.ok();
                                                },
                                                CMD_TO => {
                                                    // Handle \to command
                                                    let content = &line.as_bytes()[1..]; // Skip command byte
                                                    if let Some(space_pos) = content.iter().position(|&b| b == b' ') {
                                                        let target_nick = std::str::from_utf8(&content[..space_pos]).unwrap_or("");
                                                        let message = std::str::from_utf8(&content[space_pos+1..]).unwrap_or("");
                                                        
                                                        // Find target's address
                                                        let target_addr = {
                                                            let nick_addr_map = nick_to_addr.lock().await;
                                                            nick_addr_map.get(target_nick).cloned()
                                                        };
                                                        
                                                        if let Some(target_addr) = target_addr {
                                                            // Format whisper message for target
                                                            let whisper_msg = format!("[From {}] (whisper) {}", nickname, message);
                                                            tx.send(format!("TO_ADDR:{} {}", target_addr, whisper_msg)).ok();
                                                            
                                                            // Confirmation for sender
                                                            writer.write_all(format!("[To {}] (whisper) {}\n", target_nick, message).as_bytes()).await.ok();
                                                            writer.flush().await.ok();
                                                        } else {
                                                            // Target not found
                                                            writer.write_all(format!("Error: User '{}' not found\n", target_nick).as_bytes()).await.ok();
                                                            writer.flush().await.ok();
                                                        }
                                                    }
                                                },
                                                CMD_EXCEPT => {
                                                    // Handle \except command
                                                    let content = &line.as_bytes()[1..]; // Skip command byte
                                                    if let Some(space_pos) = content.iter().position(|&b| b == b' ') {
                                                        let except_nick = std::str::from_utf8(&content[..space_pos]).unwrap_or("");
                                                        let message = std::str::from_utf8(&content[space_pos+1..]).unwrap_or("");
                                                        
                                                        // Check if user exists
                                                        let user_exists = {
                                                            let nick_addr_map = nick_to_addr.lock().await;
                                                            nick_addr_map.contains_key(except_nick)
                                                        };
                                                        
                                                        if user_exists {
                                                            // Format message for broadcast with except tag
                                                            let except_msg = format!("[{}] (except {}) {}", nickname, except_nick, message);
                                                            tx.send(format!("EXCEPT:{} {}", except_nick, except_msg)).ok();
                                                            
                                                        } else {
                                                            // Target not found
                                                            writer.write_all(format!("Error: User '{}' not found\n", except_nick).as_bytes()).await.ok();
                                                            writer.flush().await.ok();
                                                        }
                                                    }
                                                },
                                                CMD_BAN => {
                                                    // Handle \ban command
                                                    let target_nick = std::str::from_utf8(&line.as_bytes()[1..]).unwrap_or("");
                                                    
                                                    // Find target's address
                                                    let target_addr = {
                                                        let nick_addr_map = nick_to_addr.lock().await;
                                                        nick_addr_map.get(target_nick).cloned()
                                                    };
                                                    
                                                    if let Some(target_addr) = target_addr {
                                                        // Send ban message to target
                                                        let ban_msg = format!("BAN:{} {}", target_addr, nickname);
                                                        tx.send(ban_msg).ok();
                                                        
                                                        // Confirmation for banner
                                                        writer.write_all(format!("You have banned {}\n", target_nick).as_bytes()).await.ok();
                                                        writer.flush().await.ok();
                                                        
                                                        // Announce to others
                                                        tx.send(format!("{} has been banned by {}", target_nick, nickname)).ok();
                                                    } else {
                                                        // Target not found
                                                        writer.write_all(format!("Error: User '{}' not found\n", target_nick).as_bytes()).await.ok();
                                                        writer.flush().await.ok();
                                                    }
                                                },
                                                CMD_PING => {
                                                    // Handle \ping command - just send back a PING response
                                                    writer.write_all(b"PING\n").await.ok();
                                                    writer.flush().await.ok();
                                                },
                                                _ => {}
                                            }

                                            }
                                            else if first_byte <0x20{
                                                    // when version does not match client and server
                                                    let command_str = format!("invalid command code: 0x{:02X}", first_byte);
                                                    writer.write_all(format!("{}\n", command_str).as_bytes()).await.ok();
                                                    writer.flush().await.ok();
                                            } 
                                            else {
                                                    let msg = format!("[{}] {}", nickname, line);
                                                    if msg.to_lowercase().contains("i hate professor"){
                                                        let _ = tx.send(format!("BAN:{} SERVER", addr));
                                                        continue;
                                                    }
                                                    if tx.send(msg).is_err() {
                                                        break;
                                                    }
                                            }
                                        }
                                    }
                                    Ok(None) | Err(_) => {
                                        break;
                                    }
                                }
                            }
                            // Receive message from others and send to client
                            result = rx.recv() => {
                                match result {
                                    Ok(msg) => {
                                        // Check for special message types
                                        if msg.starts_with("TO_ADDR:") {
                                            // Direct message for specific address
                                            let parts: Vec<&str> = msg.splitn(2, " ").collect();
                                            if parts.len() == 2 {
                                                let target_addr = &parts[0][8..]; // Skip "TO_ADDR:"
                                                let content = parts[1];
                                                
                                                // Only send if this client is the target
                                                if addr.to_string() == target_addr {
                                                    if writer.write_all(content.as_bytes()).await.is_err() {
                                                        break;
                                                    }
                                                    if writer.write_all(b"\n").await.is_err() {
                                                        break;
                                                    }
                                                    if writer.flush().await.is_err() {
                                                        break;
                                                    }
                                                }
                                            }
                                        } else if msg.starts_with("EXCEPT:") {
                                            // Message for everyone except specified user
                                            let parts: Vec<&str> = msg.splitn(3, " ").collect();
                                            if parts.len() >= 3 {
                                                let except_nick = &parts[0][7..]; // Skip "EXCEPT:"
                                                let content = parts[2];
                                                
                                                // Only send if this client is not the excepted user
                                                if nickname != except_nick && !msg.contains(&my_nickname_tag) {
                                                    if writer.write_all(content.as_bytes()).await.is_err() {
                                                        break;
                                                    }
                                                    if writer.write_all(b"\n").await.is_err() {
                                                        break;
                                                    }
                                                    if writer.flush().await.is_err() {
                                                        break;
                                                    }
                                                }
                                            }
                                        } else if msg.starts_with("BAN:") {

                                            // Ban message for specific client
                                            let parts: Vec<&str> = msg.splitn(2, " ").collect();
                                            if parts.len() == 2 {
                                                let target_addr = &parts[0][4..]; // Skip "BAN:"
                                                let banner_nick = parts[1];
                                                
                                                // If this client is being banned
                                                if addr.to_string() == target_addr {
                                                    let ban_msg = format!("You are banned by {}\n", banner_nick);
                                                    let _ = writer.write_all(ban_msg.as_bytes()).await;
                                                    let _ = writer.flush().await;
                                                    was_banned = true;
                                                    break; // This will terminate the client handler
                                                }
                                            }
                                        } 
                                        
                                        else if !msg.starts_with(&my_nickname_tag) {
                                            // Regular message, not from self
                                            if writer.write_all(msg.as_bytes()).await.is_err() {
                                                break;
                                            }
                                            if writer.write_all(b"\n").await.is_err() {
                                                break;
                                            }
                                            if writer.flush().await.is_err() {
                                                break;
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    // Cleanup when client disconnects
                    {
                        let mut map = nick_map.lock().await;
                        map.remove(&addr.to_string());
                        
                        let mut nick_addr_map = nick_to_addr.lock().await;
                        nick_addr_map.remove(&nickname);
                        
                        let current_count = map.len();
                        let left_message = if was_banned {
                                format!("{} is disconnected. There are {} users now", nickname, current_count)
                            } else {
                                format!("{} left the room. There are {} users now", nickname, current_count)
                            };
                        println!("{}", left_message);
                        let _ = tx.send(left_message);
                    }
                    heartbeat_task.abort();
                });
            }

            _ = shutdown_rx.recv() => {
                println!("gg~");

                let map = nickname_map.lock().await;
                for (_, nickname) in map.iter() {
                    let _ = tx.send(format!("Server is shutting down. Goodbye, {}!", nickname));
                }
                break;
            }
        }
    }

    Ok(())
}

fn is_valid_command(cmd: u8) -> bool {
    match cmd {
        CMD_LIST | CMD_TO | CMD_EXCEPT | CMD_BAN | CMD_PING => true,
        _ => false
    }
}