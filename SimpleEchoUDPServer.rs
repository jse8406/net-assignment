use std::net::UdpSocket;
use std::str;

fn main() -> std::io::Result<()> {
    let server_port = "29999";
    let server_addr = "0.0.0.0:".to_string() + server_port;

    let socket = UdpSocket::bind(server_addr.clone())?;
    println!("UDP server is ready to receive on port {}", server_port);

    let mut buf = [0; 512];
    
    loop {
        let (_size, addr) = socket.recv_from(&mut buf)?;
        let received = &buf[.._size];
        
        let msg = str::from_utf8(received).unwrap();
        let upper = msg.to_uppercase();
        
        //println!("Received({}) from {}: {}", _size, addr, msg);
        println!("Received msg from {}", addr);
        
        socket.send_to(upper.as_bytes(), addr)?;
    }
}
