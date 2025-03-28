use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let port = "11406";
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(addr)?;
    println!("Server started on port {}. Waiting for clients...", port);

    let start_time = Instant::now();
    let mut request_count = 0;

    loop {
        let (mut stream, client_addr) = listener.accept()?;
        println!("Client connected from {}", client_addr);

        loop {
            let mut buffer = [0; 512];
            let size = match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Client disconnected.");
                    break; // 내부 루프만 탈출, 외부 루프는 계속 (다음 클라이언트 대기)
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

            // 클라이언트가 명시적으로 "OPT5"를 보내면 종료 신호로 처리
            if msg == "OPT5" {
                println!("Client requested to exit.");
                break; // 클라이언트 처리 종료, 다음 클라이언트 받기 위해 외부 루프는 유지
            }

            // 응답 처리
            let reply = if msg.starts_with("OPT1:") {
                let content = msg.strip_prefix("OPT1:").unwrap_or("");
                content.to_uppercase()
            } else if msg == "OPT2" {
                let elapsed = start_time.elapsed();
                format!("run time = {02}:{:02}:{:02}", elapsed.as_secs() / 3600, (elapsed.as_secs() / 60) % 60, elapsed.as_secs() % 60)
            } else if msg == "OPT3" {
                format!("client IP = {}, port = {}", client_addr.ip(), client_addr.port())
            } else if msg == "OPT4" {
                format!("requests served = {}", request_count)
            } else {
                "Invalid command.".to_string()
            };

            stream.write_all(reply.as_bytes())?;
        }
    }
}
