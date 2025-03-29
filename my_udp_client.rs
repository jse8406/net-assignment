use std::net::UdpSocket;
use std::str;

fn main() -> std::io::Result<()> {
    let server_ip = "nsl2.cau.ac.kr";
    let server_port = "29999";
    let server_addr = format!("{}:{}", server_ip, server_port);

    let socket = UdpSocket::bind("0.0.0.0:0")?;

    println!("Client is running on port {}",
                socket.local_addr().unwrap().port());

    let mut input = String::from("");

    println!("Input lowercase sentence: ");
    std::io::stdin().read_line(&mut input)?;
    let msg = input.trim_end();

    socket.send_to(msg.as_bytes(), server_addr)?;
    //println!("Sent ({}) : {msg}", msg.len());

    let mut buf = [0; 1024];
    
    let (_size, _src) = socket.recv_from(&mut buf)?;

    let received = str::from_utf8(&buf[.._size]).unwrap();
    println!("Reply from server: {}", received);

    Ok(())
}

