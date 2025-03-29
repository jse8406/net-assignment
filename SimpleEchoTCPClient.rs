// SEUNG EON JIN 20201406
use std::net::TcpStream;
use std::io::{Read, Write};
use std::str::from_utf8;
use std::io;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    // let server_ip = "nsl2.cau.ac.kr";
    let server_ip = "127.0.0.1";
    let server_port = "11406";
    let server_addr = format!("{}:{}", server_ip, server_port);

    // Create TCP connection to server
    let mut stream = TcpStream::connect(server_addr.clone())?;

    println!(
        "Connected to server at {} from client at {}",
        stream.peer_addr().unwrap().to_string(),
        stream.local_addr().unwrap().to_string()
    );

    loop {
        // Print options
        println!("\n--- Menu ---");
        println!("1) Convert text to UPPER-case letters");
        println!("2) Ask how long the server has been running (HH:MM:SS)");
        println!("3) Ask what the IP and port of the client are");
        println!("4) Ask how many requests the server has handled so far");
        println!("5) Exit client program");

        println!("Select option (1~5): ");

        // Get option from user
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();
        
        // Here, the message header is OPT + # of option. But in option 1, the message contains the text which will be converted to upper case and others have the only message headers.
        let msg_to_send = match choice {
            "1" => {
                // option 1, message is header + text
                loop {
                    print!("Enter text to convert to UPPER-case: ");
                    io::Write::flush(&mut io::stdout())?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let trimmed = input.trim_end();

                    // check if the text is only composed with english or number
                    if trimmed.chars().all(|c| c.is_ascii_alphabetic() || c.is_ascii_whitespace() || c.is_ascii_digit()) {
                        break format!("OPT1{}", trimmed);
                    } else {
                        println!(" Only English letters, numbers and spaces are allowed. Please try again.");
                    }
                }
            }
            // others message is only header
            "2" => "OPT2".to_string(),
            "3" => "OPT3".to_string(),
            "4" => "OPT4".to_string(),
            // exit program => repeat the menu until getting command option 5
            "5" => {
                println!("Exiting program.");
                break;
            }
            // When get wrong option (not in 1~5)
            _ => {
                println!("Invalid option. Try again.");
                continue;
            }
        };

        
        // check the time before sending the command
        let start_time = Instant::now();
        
        // Send message to server
        // OPT1:{text} or OPT2, OPT3, OPT4
        stream.write_all(msg_to_send.as_bytes())?;

        // Get response from the server
        let mut buffer = [0; 512];
        let size = stream.read(&mut buffer)?;
        let reply = from_utf8(&buffer[..size]).unwrap_or("[Invalid UTF-8 reply]");

        // after receiving the reply
        let elapsed = start_time.elapsed();
        println!("Reply from server: {}", reply);
        println!("RTT = {:.3} ms", elapsed.as_secs_f64() * 1000.0); // 1s = 1000ms
    }

    Ok(())
}
