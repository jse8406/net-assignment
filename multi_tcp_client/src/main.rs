// SEUNG EON JIN 20201406
use std::net::TcpStream;
use std::io::{self, Read, Write};
use std::str::from_utf8;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Instant;
use ctrlc;
use std::process;

fn main() -> std::io::Result<()> {
    let server_ip = "localhost";
    // let server_ip = "nsl5.cau.ac.kr";
    let server_port = "11406";
    let server_addr = format!("{}:{}", server_ip, server_port);

    // Create TCP connection to server
    let stream = TcpStream::connect(server_addr.clone())?;

    println!(
        "Connected to server at {} from client at {}",
        stream.peer_addr().unwrap().to_string(),
        stream.local_addr().unwrap().to_string()
    );

    let stream_arc = Arc::new(Mutex::new(stream));

    // Flag to notify server watcher thread to exit
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_for_server = Arc::clone(&stop_flag);

    // Ctrl+C interrupt handling
    {
        let stream_clone = Arc::clone(&stream_arc);
        ctrlc::set_handler(move || {
            let _ = stream_clone.lock().unwrap().shutdown(std::net::Shutdown::Both);
            println!("\nBye bye~");
            process::exit(0);
        }).expect("Error setting Ctrl-C handler");
    }

    // Clone TcpStream for server watcher thread
    let stream_for_server = {
        let guard = stream_arc.lock().unwrap();
        guard.try_clone().expect("Failed to clone TcpStream")
    };

    // Server watcher thread: monitors if the server disconnects
    thread::spawn(move || {
        let mut buf = [0; 1];
        loop {
            if stop_flag_for_server.load(Ordering::Relaxed) {
                // Exit when stop flag is set
                break;
            }

            let result = stream_for_server.peek(&mut buf);
            match result {
                Ok(0) => {
                    println!("\nServer disconnected. Terminating.");
                    process::exit(0);
                }
                Ok(_) => {
                    // Server is alive, check again after a short sleep
                    thread::sleep(std::time::Duration::from_millis(500));
                }
                Err(ref e) => {
                    if stop_flag_for_server.load(Ordering::Relaxed) {
                        // If shutdown initiated, ignore error and exit quietly
                        break;
                    }

                    if let Some(code) = e.raw_os_error() {
                        #[cfg(windows)]
                        if code == 10053 || code == 10054 {
                            break;
                        }
                        #[cfg(unix)]
                        if code == 104 {
                            break;
                        }
                    }

                    // Unexpected error (not normal shutdown)
                    println!("\nUnexpected connection error: {}", e);
                    process::exit(1);
                }
            }
        }
    });

    // Main thread: User input handling
    loop {
        // Print options
        println!("\n--- Menu ---");
        println!("1) Convert text to UPPER-case letters");
        println!("2) Ask how long the server has been running for since server started (HH:MM:SS)");
        println!("3) Ask what the IP and port of the client are");
        println!("4) Ask how many requests the server has handled so far");
        println!("5) Exit client program");

        print!("Select option (1~5): ");
        io::Write::flush(&mut io::stdout())?;

        // Get option from user
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        // Prepare message to send
        let msg_to_send = match choice {
            "1" => {
                // Option 1: user provides additional text input
                loop {
                    print!("Enter text to convert to UPPER-case: ");
                    io::Write::flush(&mut io::stdout())?;
                    let mut input = String::new();
                    if io::stdin().read_line(&mut input)? == 0 {
                        println!("EOF : invalid input...");
                        continue;
                    }
                    let trimmed = input.trim_end();
                    if trimmed.chars().all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace() || c.is_ascii_digit()) {
                        break format!("OPT1{}", trimmed);
                    } else {
                        println!("Only English letters, numbers and spaces are allowed. Please try again.");
                    }
                }
            }
            // Options 2~4: simple header message
            "2" => "OPT2".to_string(),
            "3" => "OPT3".to_string(),
            "4" => "OPT4".to_string(),
            "5" => {
                // Exit program
                stop_flag.store(true, Ordering::Relaxed); // notify watcher thread

                let _ = stream_arc.lock().unwrap().write_all(b"OPT5");
                let _ = stream_arc.lock().unwrap().shutdown(std::net::Shutdown::Both);

                println!("Bye bye~");
                break;
            }
            _ => {
                println!("Invalid option. Try again.");
                continue;
            }
        };

        // Check time before sending the command
        let start_time = Instant::now();

        // Send message to server
        let write_result = stream_arc.lock().unwrap().write_all(msg_to_send.as_bytes());
        if let Err(e) = write_result {
            if handle_io_error(&e, "Write") {
                break;
            }
        }

        // Get response from server
        let mut buffer = [0; 512];
        let size = match stream_arc.lock().unwrap().read(&mut buffer) {
            Ok(0) => {
                println!("Server closed the connection.");
                break;
            }
            Ok(n) => n,
            Err(e) => {
                if handle_io_error(&e, "Read") {
                    break;
                }
                0
            }
        };

        // Prevent failure of utf-8 decoding
        let reply = from_utf8(&buffer[..size]).unwrap_or("[Invalid UTF-8 reply]");

        // Check RTT after receiving reply
        let elapsed = start_time.elapsed();

        // Print the reply and RTT
        println!("Reply from server: {}", reply);
        println!("RTT = {:.3} ms", elapsed.as_secs_f64() * 1000.0); // 1s = 1000ms
    }

    Ok(())
}

// Handle IO errors
fn handle_io_error(e: &std::io::Error, context: &str) -> bool {
    let is_server_terminated = match e.raw_os_error() {
        Some(code) => {
            #[cfg(windows)]
            {
                code == 10053
            }
            #[cfg(unix)]
            {
                code == 104
            }
        }
        None => false,
    };

    if is_server_terminated {
        println!("server terminated");
    } else {
        eprintln!("{} failed: {}", context, e);
    }

    true
}
