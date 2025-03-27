use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

fn main() -> std::io::Result<()> {
    let server_ip = "nsl2.cau.ac.kr";
    let server_port = "39999";
    let server_addr = format!("{}:{}", server_ip, server_port);

    let mut stream = TcpStream::connect(server_addr.clone())?;

    println!("Connected to server at {} from client at {}", 
                stream.peer_addr().unwrap().to_string(),
                stream.local_addr().unwrap().to_string());

    let mut input = String::from("");

    println!("Input lowercase sentence: ");
    std::io::stdin().read_line(&mut input)?;
    let msg = input.trim_end();

    stream.write(msg.as_bytes())?;
    //println!("Sent ({}) : {msg}", msg.len());

    let mut buffer = [0; 512];

    let _size = stream.read(&mut buffer)?;

    let text2 = from_utf8(&buffer).unwrap();
    //println!("Reply({_size}) : {text2}");
    println!("Reply from server: {}", text2);

    Ok(())
}

