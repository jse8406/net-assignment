// SEUNG EON JIN 20201406
use std::net::UdpSocket;
use std::str::from_utf8;
use std::time::{Duration, Instant};
use std::io::{self};

fn main() -> std::io::Result<()> {
    let server_ip = "127.0.0.1";

    let server_port = "11406";
    let server_addr = format!("{}:{}", server_ip, server_port);

    // Bind to a random port (auto-assigned)
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    println!("UDP client running on {}", socket.local_addr()?);

    loop {
        println!("\n--- Menu ---");
        println!("1) Convert text to UPPER-case letters");
        println!("2) Ask how long the server has been running (HH:MM:SS)");
        println!("3) Ask what the IP and port of the client are");
        println!("4) Ask how many requests the server has handled so far");
        println!("5) Exit client program");

        print!("Select option (1~5): ");
        io::Write::flush(&mut io::stdout())?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();

        let msg_to_send = match choice {
            "1" => {
                loop {
                    print!("Enter text to convert to UPPER-case: ");
                    io::Write::flush(&mut io::stdout())?;
                    let mut input = String::new();
                    if io::stdin().read_line(&mut input)? == 0 {
                        println!("EOF : invalid input...");
                        continue;
                    }
                    let trimmed = input.trim_end();

                    if trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace()) {
                        break format!("OPT1{}", trimmed);
                    } else {
                        println!("Only English letters, numbers, and spaces are allowed. Please try again.");
                    }
                }
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

        // RTT 측정 시작
        let start_time = Instant::now();
        socket.send_to(msg_to_send.as_bytes(), &server_addr)?;

        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((size, _src)) => {
                let expected_addr = server_addr.parse().unwrap(); // SocketAddr
                // only receive the expected address message, ignore another
                if src != expected_addr {
                    println!("Received packet from unexpected source: {}", src);
                    continue;
                }
                let reply = from_utf8(&buf[..size]).unwrap_or("[Invalid UTF-8 reply]");
                let elapsed = start_time.elapsed();
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
