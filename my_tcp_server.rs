// SEUNG EON JIN 20201406
use std::net::{TcpListener};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let port = "11406";
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr)?;
    println!("Server started on port {}. Waiting for clients...", port);

    let start_time = Instant::now(); // Record server start time
    let mut request_count = 0;       // Count the total number of served requests

    loop {
        let (mut stream, client_addr) = listener.accept()?;
        println!("Client connected from {}", client_addr);

        loop {
            let mut buffer = [0; 512];
            let size = match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Client disconnected.");
                    break; // If client disconnected, wait for the next one (until manually terminated with Ctrl+C)
                }
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            };

            let msg = from_utf8(&buffer[..size]).unwrap_or("");
            println!("Received: {}", msg);
            request_count += 1;

            // If the client explicitly sends "OPT5", treat it as a termination request
            if msg == "OPT5" {
                println!("Client requested to exit.");
                break; // End current client session, return to accept the next client
            }

            // Handle client request based on the message header
            let reply = if msg.starts_with("OPT1") {
                // OPT1: Convert the rest of the message to uppercase
                let content = msg.strip_prefix("OPT1").unwrap_or("");
                content.to_uppercase()
            } else if msg == "OPT2" {
                // OPT2: Return server uptime (HH:MM:SS)
                let elapsed = start_time.elapsed();
                format!(
                    "run time = {:02}:{:02}:{:02}",
                    elapsed.as_secs() / 3600,
                    (elapsed.as_secs() / 60) % 60,
                    elapsed.as_secs() % 60
                )
            } else if msg == "OPT3" {
                // OPT3: Return client's IP and port
                format!("client IP = {}, port = {}", client_addr.ip(), client_addr.port())
            } else if msg == "OPT4" {
                // OPT4: Return total number of requests served
                format!("requests served = {}", request_count)
            } else {
                // Handle unexpected or invalid message
                "Invalid message.".to_string()
            };

            // Send the reply to the client
            stream.write_all(reply.as_bytes())?;
        }
    }
}
