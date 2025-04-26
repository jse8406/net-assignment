// SEUNG EON JIN 20201406
use std::net::TcpStream;
use std::io::{self, Read, Write};
use std::str::from_utf8;
use std::time::Duration;
use std::time::Instant;
use std::sync::{Arc, Mutex};
use ctrlc;
use std::process;
use std::thread;

fn main() -> std::io::Result<()> {
    // let server_ip = "nsl5.cau.ac.kr";
    let server_ip = "localhost";
    let server_port = "11406";
    let server_addr = format!("{}:{}", server_ip, server_port);

    // Create TCP connection to server
    let stream = TcpStream::connect(server_addr.clone())?;
    stream.set_read_timeout(Some(Duration::from_secs(1)))?; // Reduced timeout
    stream.set_nonblocking(true)?; // Make socket non-blocking

    println!(
        "Connected to server at {} from client at {}",
        stream.peer_addr().unwrap().to_string(),
        stream.local_addr().unwrap().to_string()
    );

    let stream = Arc::new(Mutex::new(stream));
    let stream_clone = Arc::clone(&stream);
    
    // Create a flag to indicate server termination
    let server_terminated = Arc::new(Mutex::new(false));
    let server_terminated_clone = Arc::clone(&server_terminated);

    // Spawn a thread to listen for server termination
    let stream_clone_for_listener = Arc::clone(&stream);
    thread::spawn(move || {
        let mut buffer = [0; 512];
        loop {
            match stream_clone_for_listener.lock().unwrap().read(&mut buffer) {
                Ok(0) => {
                    *server_terminated_clone.lock().unwrap() = true;
                    println!("\n[Notice] Server connection lost. Exiting client...");
                    process::exit(0);
                },
                Ok(n) => {
                    let msg = from_utf8(&buffer[..n]).unwrap_or("");
                    if msg.contains("[Server terminated]") {
                        *server_terminated_clone.lock().unwrap() = true;
                        println!("\n[Notice] Server has terminated. Exiting client...");
                        process::exit(0);
                    }
                    // If we got a regular message, just ignore it as it will be handled by the main thread
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // No data available right now, sleep for a bit
                    thread::sleep(Duration::from_millis(100));
                },
                Err(_) => {
                    *server_terminated_clone.lock().unwrap() = true;
                    println!("\n[Notice] Connection error. Exiting client...");
                    process::exit(0);
                }
            }
        }
    });

    ctrlc::set_handler(move || {
        let _ = stream_clone.lock().unwrap().shutdown(std::net::Shutdown::Both);
        println!("\nBye bye~");
        process::exit(0);
    }).expect("Error setting Ctrl-C handler");
    
    loop {
        if *server_terminated.lock().unwrap() {
            break;
        }

        // Print options
        println!("\n--- Menu ---");
        println!("1) Convert text to UPPER-case letters");
        println!("2) Ask how long the server has been running for since server started (HH:MM:SS)");
        println!("3) Ask what the IP and port of the client are");
        println!("4) Ask how many requests the server has handled so far");
        println!("5) Exit client program");

        println!("Select option (1~5): ");
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        if *server_terminated.lock().unwrap() {
            break;
        }
        
        let msg_to_send = match choice {
            "1" => {
                // Option 1, message is header + text
                print!("Enter text to convert to UPPER-case: ");
                io::Write::flush(&mut io::stdout())?;
                let mut input = String::new();
                
                match io::stdin().read_line(&mut input) {
                    Ok(0) | Err(_) => {
                        println!("Invalid input, try again.");
                        continue;
                    },
                    Ok(_) => {}
                }

                if *server_terminated.lock().unwrap() {
                    break;
                }
                
                let trimmed = input.trim_end();

                if !trimmed.chars().all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace() || c.is_ascii_digit()) {
                    println!("Only English letters, numbers and spaces are allowed. Please try again.");
                    continue;
                }
                format!("OPT1{}", trimmed)
            }
            "2" => "OPT2".to_string(),
            "3" => "OPT3".to_string(),
            "4" => "OPT4".to_string(),
            "5" => {
                println!("Exiting program.");
                break;
            }
            _ => {
                println!("Invalid option. Try again.");
                continue;
            }
        };

        if *server_terminated.lock().unwrap() {
            break;
        }

        let start_time = Instant::now();
        
        // Send message to server with timeout handling
        match stream.lock().unwrap().write_all(msg_to_send.as_bytes()) {
            Ok(_) => {},
            Err(_) => {
                println!("Failed to send message to server.");
                break;
            }
        }

        // Get response from the server with timeout
        let mut buffer = [0; 512];
        let mut response_received = false;
        let timeout = Duration::from_secs(5);
        let start = Instant::now();

        while !response_received && start.elapsed() < timeout {
            match stream.lock().unwrap().read(&mut buffer) {
                Ok(0) => {
                    println!("Server closed the connection.");
                    return Ok(());
                }
                Ok(n) => {
                    let reply = from_utf8(&buffer[..n]).unwrap_or("[Invalid UTF-8 reply]");
                    let elapsed = start_time.elapsed();
                    println!("Reply from server: {}", reply);
                    println!("RTT = {:.3} ms", elapsed.as_secs_f64() * 1000.0);
                    response_received = true;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(e) => {
                    eprintln!("Read failed: {}", e);
                    return Ok(());
                }
            }
        }

        if !response_received {
            println!("No response from server after 5 seconds.");
            break;
        }
    }

    Ok(())
}
