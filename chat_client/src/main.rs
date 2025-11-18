//20201406 SEUNG EON JIN

const CMD_LIST: u8   = 0x01;
const CMD_TO: u8     = 0x02;
const CMD_EXCEPT: u8 = 0x03;
const CMD_BAN: u8    = 0x04;
const CMD_PING: u8   = 0x05;
// const CMD_TEST: u8   = 0x06;

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::Mutex,
    signal,
    time::Instant,
};
use std::sync::Arc;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <nickname>");
        std::process::exit(1);
    }

    let nickname = args[1].clone();
    let server_ip = "127.0.0.1";
    let server_port = "11406";
    let server_addr = format!("{}:{}", server_ip, server_port);
    let stream = TcpStream::connect(&server_addr).await?;
    let (reader, writer) = stream.into_split();

    let reader = Arc::new(Mutex::new(Some(BufReader::new(reader))));
    let reader_for_drop = Arc::clone(&reader);

    let writer = Arc::new(Mutex::new(writer));
    let writer_for_ctrlc = Arc::clone(&writer);

    // For ping command
    let ping_start_time = Arc::new(Mutex::new(None::<Instant>));
    let ping_start_time_for_reader = Arc::clone(&ping_start_time);

    let mut server_reader = reader.lock().await.take().unwrap().lines();

    // Check if the chatting room is full
    if let Some(line) = server_reader.next_line().await? {
        println!("{}", line);
        if line.contains("chatting room full") {
            return Ok(());
        }
    }

    // // Enter nickname
    // let mut stdin = BufReader::new(tokio::io::stdin()).lines();
    // let nickname = stdin.next_line().await?.unwrap_or("Anonymous".to_string());

    {
        let mut w = writer.lock().await;
        w.write_all(nickname.as_bytes()).await?;
        w.write_all(b"\n").await?;
    }
    

    // input task
    let mut stdin = BufReader::new(tokio::io::stdin()).lines();

    let shutdown_trigger = Arc::new(Mutex::new(false));
    let shutdown_trigger_clone = Arc::clone(&shutdown_trigger);

    // Store the stdin task handle to abort it when Ctrl+C is pressed
    let stdin_task = tokio::spawn(async move {
        loop {
            if *shutdown_trigger_clone.lock().await {
                break;
            }
    match stdin.next_line().await {
        Ok(Some(line)) => {
            let encoded = if line.starts_with('\\') {
                match encode_command(&line) {
                    Some(buf) => {
                        // For ping command, start the timer
                        if line.trim() == r"\ping" {
                            *ping_start_time.lock().await = Some(Instant::now());
                        }
                        buf
                    }
                    None => {
                        println!("invalid command");
                        continue;
                    }
                }
            } else {
                line.into_bytes()
            };

            let mut w = writer.lock().await;
            if w.write_all(&encoded).await.is_err() {
                break;
            }
            w.write_all(b"\n").await.ok();
        }
        _ => break,
    }

        }
    });

    // Get message, Ctrl + C control
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("gg~");
            *shutdown_trigger.lock().await = true;
            
            // Abort the stdin task to prevent waiting for Enter
            stdin_task.abort();
            
            // Just properly close the TCP connection without sending a special message
            let mut w = writer_for_ctrlc.lock().await;
            let _ = w.shutdown().await;
            drop(w);
            drop(reader_for_drop);
            
            // Force program exit after cleanup
            std::process::exit(0);
        }
        _ = async {
            while let Some(result) = server_reader.next_line().await.transpose() {
                match result {
                    Ok(msg) => {
                        // Check if this is a response to ping
                        if msg == "PING" {
                            // Calculate RTT
                          let start_time_option = {
                                let mut ping_time = ping_start_time_for_reader.lock().await;
                                let start_time = *ping_time;
                                *ping_time = None;
                                start_time 
                            };
                            
                   
                            if let Some(start_time) = start_time_option {
                                let elapsed = start_time.elapsed();
                                let rtt_ms = elapsed.as_secs_f64() * 1000.0;
                                println!("RTT: {} ms", rtt_ms);
                            }
                        } 
                        // Check if you're being banned
                        else if msg.starts_with("You are banned by") {
                            println!("{}", msg);
                            *shutdown_trigger.lock().await = true;
                            std::process::exit(0);
                        }
                        else {
                            println!("{}", msg);
                        }
                    },
                    // when server disconnected
                    Err(_) => {
                        break;
                    }
                }
            }
            // If we get here, the server has closed the connection
            println!("[Client] Server closed the connection");
            *shutdown_trigger.lock().await = true;
            std::process::exit(0);
        } => {}
    }

    Ok(())
}

fn encode_command(input: &str) -> Option<Vec<u8>> {
    let mut parts = input.trim().split_whitespace();
    let command = parts.next()?; // e.g., \to

    match command {
        r"\list" => Some(vec![CMD_LIST]),
        r"\ping" => Some(vec![CMD_PING]),
        r"\ban" => {
            let target = parts.next()?;
            let mut msg = vec![CMD_BAN];
            msg.extend_from_slice(target.as_bytes());
            Some(msg)
        }
        r"\to" => {
            let target = parts.next()?;
            let message: String = parts.collect::<Vec<_>>().join(" ");
            let mut msg = vec![CMD_TO];
            msg.extend_from_slice(target.as_bytes());
            msg.push(b' '); // separate nickname and message
            msg.extend_from_slice(message.as_bytes());
            Some(msg)
        }
        r"\except" => {
            let target = parts.next()?;
            let message: String = parts.collect::<Vec<_>>().join(" ");
            let mut msg = vec![CMD_EXCEPT];
            msg.extend_from_slice(target.as_bytes());
            msg.push(b' ');
            msg.extend_from_slice(message.as_bytes());
            Some(msg)
        }
        //  r"\test" => {
        //     // for testing server invalid command
        //     let message: String = parts.collect::<Vec<_>>().join(" ");
        //     let mut msg = vec![CMD_TEST];
        //     if !message.is_empty() {
        //         msg.push(b' ');
        //         msg.extend_from_slice(message.as_bytes());
        //     }
        //     Some(msg)
        // },
        _ => None, // invalid command
    }
}