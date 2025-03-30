// SEUNG EON JIN 20201406
use std::net::UdpSocket;
use std::str::from_utf8;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let port = "11406";
    let addr = format!("0.0.0.0:{}", port);
    let socket = UdpSocket::bind(&addr)?;
    println!("UDP server started on port {}. Waiting for messages...", port);

    let start_time = Instant::now(); // Track server uptime
    let mut request_count = 0;

    let mut buffer = [0; 1024];

    loop {
        let (size, src_addr) = match socket.recv_from(&mut buffer) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to receive: {}", e);
                continue;
            }
        };

        let msg = match from_utf8(&buffer[..size]) {
            Ok(text) => text,
            Err(_) => {
                eprintln!("Invalid UTF-8 received from {}", src_addr);
                continue;
            }
        };

        println!("Received from {}: {}", src_addr, msg);
        request_count += 1;

        // Handle commands
        let reply = if msg.starts_with("OPT1") {
            let content = msg.strip_prefix("OPT1").unwrap_or("");
            content.to_uppercase()
        } else if msg == "OPT2" {
            let elapsed = start_time.elapsed();
            format!(
                "run time = {:02}:{:02}:{:02}",
                elapsed.as_secs() / 3600,
                (elapsed.as_secs() / 60) % 60,
                elapsed.as_secs() % 60
            )
        } else if msg == "OPT3" {
            format!("client IP = {}, port = {}", src_addr.ip(), src_addr.port())
        } else if msg == "OPT4" {
            format!("requests served = {}", request_count)
        } else if msg == "OPT5" {
            // 클라이언트 종료 메시지지만, UDP 서버는 연결 유지 개념이 없으므로 무시
            println!("Client sent OPT5 (exit request)");
            "Goodbye.".to_string()
        } else {
            "Invalid message.".to_string()
        };

        if let Err(e) = socket.send_to(reply.as_bytes(), &src_addr) {
            eprintln!("Failed to send response to {}: {}", src_addr, e);
        }
    }
}
