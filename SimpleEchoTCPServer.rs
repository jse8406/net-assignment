use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let port = "11406";
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr)?;
    println!("Server started on port {}. Waiting for client...", port);

    // 서버 시작 시간 저장
    let start_time = Instant::now();
    let mut request_count = 0;

    // 클라이언트 1명만 받는다
    let (mut stream, client_addr) = listener.accept()?;
    println!("Client connected from {}", client_addr);

    loop {
        let mut buffer = [0; 512];
        let size = match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Client disconnected.");
                break;
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

        let reply = if msg.starts_with("OPT1:") {
            let content = msg.trim_start_matches("OPT1:");
            content.to_uppercase()
        } else if msg == "OPT2" {
            let elapsed = start_time.elapsed();
            let h = elapsed.as_secs() / 3600;
            let m = (elapsed.as_secs() % 3600) / 60;
            let s = elapsed.as_secs() % 60;
            format!("{:02}:{:02}:{:02}", h, m, s)
        } else if msg == "OPT3" {
            format!("Your IP: {}, Port: {}", client_addr.ip(), client_addr.port())
        } else if msg == "OPT4" {
            format!("Total requests so far: {}", request_count)
        } else {
            "Invalid command.".to_string()
        };

        stream.write_all(reply.as_bytes())?;
    }

    Ok(())
}
