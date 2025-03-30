// SEUNG EON JIN 20201406
use std::net::{UdpSocket, ToSocketAddrs}; 
use std::str::from_utf8;
use std::time::{Instant};
use std::io::{self};
use std::time::Duration;


fn main() -> std::io::Result<()> {
    // Set server IP and port
    let server_ip = "nsl5.cau.ac.kr"; // For remote server
    // let server_ip = "127.0.0.1"; // For local testing
    let server_port = "31406";
    let server_addr = format!("{}:{}", server_ip, server_port);

    // Bind to a random available port (client side)
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    println!("UDP client running on {}", socket.local_addr()?);
    socket.set_read_timeout(Some(Duration::new(5, 0)))?; // Set read timeout to 5 seconds
    loop {
        // Show user menu
        println!("\n--- Menu ---");
        println!("1) Convert text to UPPER-case letters");
        println!("2) Ask how long the server has been running (HH:MM:SS)");
        println!("3) Ask what the IP and port of the client are");
        println!("4) Ask how many requests the server has handled so far");
        println!("5) Exit client program");

        // Prompt user for menu selection
        print!("Select option (1~5): ");
        io::Write::flush(&mut io::stdout())?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        // Construct message to send based on selected option
        let msg_to_send = match choice {
            "1" => {
                // Prompt user to input text to convert
                loop {
                    print!("Enter text to convert to UPPER-case: ");
                    io::Write::flush(&mut io::stdout())?;
                    let mut input = String::new();
                    if io::stdin().read_line(&mut input)? == 0 {
                        println!("EOF : invalid input...");
                        continue;
                    }
                    let trimmed = input.trim_end();

                    // Only allow alphanumeric characters and spaces
                    if trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace()) {
                        break format!("OPT1{}", trimmed);
                    } else {
                        println!("Only English letters, numbers, and spaces are allowed. Please try again.");
                    }
                }
            }
            "2" => "OPT2".to_string(), // Request server uptime
            "3" => "OPT3".to_string(), // Request client IP and port
            "4" => "OPT4".to_string(), // Request total number of requests served
            "5" => {
                // Terminate client program
                println!("Exiting program.");
                break;
            }
            _ => {
                // Invalid option
                println!("Invalid option. Try again.");
                continue;
            }
        };

        // Start RTT (Round Trip Time) timer
        let start_time = Instant::now();

        // Send the message to the server
        socket.send_to(msg_to_send.as_bytes(), &server_addr)?;

        let mut buf = [0; 1024]; // Buffer for receiving response

        // Receive response from server
        // Receive response from server
        match socket.recv_from(&mut buf) {
            Ok((size, src)) => {
                // DNS resolve to ipv4 address
                // In Linux, cannot parse the DNS:PORT format directly
                // So, we need to resolve the server address to get the expected address
                let mut addrs_iter = server_addr.to_socket_addrs()?;
                let expected_addr = match addrs_iter.next() {
                    Some(addr) => addr,
                    None => {
                        eprintln!("Could not resolve server address '{}'", server_addr);
                        break;
                    }
                };

                // Only accept response from expected server
                if src != expected_addr {
                    println!("Received packet from unexpected source: {}", src);
                    continue;
                }

                // Parse UTF-8 response from server
                let reply = from_utf8(&buf[..size]).unwrap_or("[Invalid UTF-8 reply]");
                let elapsed = start_time.elapsed(); // Stop RTT timer

                // Display server reply and RTT
                println!("Reply from server: {}", reply);
                println!("RTT = {:.3} ms", elapsed.as_secs_f64() * 1000.0);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                println!("Read timed out after 5 seconds.");
                break;
            }
            Err(e) => {
                eprintln!("Failed to receive reply: {}", e);
                break;
            }
        }
    }

    Ok(())
}
