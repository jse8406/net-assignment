// SEUNG EON JIN 20201406
use std::net::UdpSocket;
use std::str::from_utf8;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    // Set server port number
    let port = "31406";
    let addr = format!("0.0.0.0:{}", port);

    // Bind UDP socket to the specified port
    let socket = UdpSocket::bind(&addr)?;
    println!("UDP server started on port {}. Waiting for messages...", port);

    let start_time = Instant::now(); // Track server uptime
    let mut request_count = 0;       // Count the number of processed requests

    let mut buffer = [0; 1024];      // Buffer for receiving client messages

    loop {
        // Wait to receive data from any client
        let (size, src_addr) = match socket.recv_from(&mut buffer) {
            Ok(data) => data,
            Err(e) => {
                // If receive fails, print error and continue to next loop
                eprintln!("Failed to receive: {}", e);
                continue;
            }
        };

        // Convert received bytes to UTF-8 string
        let msg = match from_utf8(&buffer[..size]) {
            Ok(text) => text,
            Err(_) => {
                // Invalid UTF-8 received, skip this message
                eprintln!("Invalid UTF-8 received from {}", src_addr);
                continue;
            }
        };

        println!("Received from {}: {}", src_addr, msg);
        request_count += 1;

        // Handle client request based on the message content
        let reply = if msg.starts_with("OPT1") {
            // OPT1: Convert the message content to uppercase
            let content = msg.strip_prefix("OPT1").unwrap_or("");
            content.to_uppercase()
        } else if msg == "OPT2" {
            // OPT2: Return server uptime in HH:MM:SS format
            let elapsed = start_time.elapsed();
            format!(
                "run time = {:02}:{:02}:{:02}",
                elapsed.as_secs() / 3600,
                (elapsed.as_secs() / 60) % 60,
                elapsed.as_secs() % 60
            )
        } else if msg == "OPT3" {
            // OPT3: Return the client's IP address and port
            format!("client IP = {}, port = {}", src_addr.ip(), src_addr.port())
        } else if msg == "OPT4" {
            // OPT4: Return the total number of requests handled so far
            format!("requests served = {}", request_count)
        } else if msg == "OPT5" {
            // OPT5: Exit request from client (UDP has no connection, so just acknowledge)
            println!("Client sent OPT5 (exit request)");
            "Goodbye.".to_string()
        } else {
            // Unknown or invalid message
            "Invalid message.".to_string()
        };

        // Send the response back to the client
        if let Err(e) = socket.send_to(reply.as_bytes(), &src_addr) {
            eprintln!("Failed to send response to {}: {}", src_addr, e);
        }
    }
}
