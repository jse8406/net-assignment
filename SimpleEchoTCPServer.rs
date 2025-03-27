
use std::net::TcpListener;
use std::io::{Read, Write};
use std::str::from_utf8;

fn main() -> std::io::Result<()> {
    let server_port = "39999";
    let server_addr = "0.0.0.0:".to_string() + server_port;

    let listener = TcpListener::bind(server_addr.clone())?;
    println!("TCP server is ready to receive on port {}", server_port);

    let mut data = [0; 512];

    loop {
        let (mut stream, _addr) = listener.accept()?;

        let _size = stream.read(&mut data)?;

        let text = from_utf8(&data[0.._size]).unwrap();
        //println!("Received({_size}) : {}", text);
        println!("Connection request from {}", _addr);

        let _ = stream.write(text.to_uppercase().as_bytes());
    }
}

