    // SEUNG EON JIN 20201406
    use std::net::TcpListener;
    use std::io::{Read, Write};
    use std::str::from_utf8;
    use std::time::{Instant, Duration};
    use std::thread;
    use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
    use std::collections::HashMap;
    use chrono::{Local, Timelike};
    use ctrlc;
    use std::process;

    fn format_time() -> String {
        let now = Local::now();
        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second())
    }

    fn main() -> std::io::Result<()> {
        let port = "11406";
        let addr = format!("0.0.0.0:{}", port);
        let listener = TcpListener::bind(addr)?;
        println!("Server started on port {}. Waiting for clients...", port);

        let start_time = Instant::now();
        let request_count = Arc::new(Mutex::new(0));
        let client_id_counter = Arc::new(Mutex::new(1));
        let clients: Arc<Mutex<HashMap<usize, std::net::TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(AtomicBool::new(true));

        // Handle Ctrl+C on server
        {
            let clients = Arc::clone(&clients);
            let running = Arc::clone(&running);
            ctrlc::set_handler(move || {
                println!("\n[Time: {}] Ctrl+C detected. Shutting down server.", format_time());
                running.store(false, Ordering::SeqCst);
                let mut lock = clients.lock().unwrap();
                for (_id, stream) in lock.drain() {
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                }
                println!("Bye bye~");
                process::exit(0);
            }).expect("Error setting Ctrl-C handler");
        }

        // Background thread: print number of clients every 10 seconds
        {
            let clients = Arc::clone(&clients);
            let running = Arc::clone(&running);
            let count = clients.lock().unwrap().len();
            println!("[Time: {}] Number of clients connected = {}", format_time(), count);
            thread::spawn(move || {
                let count = clients.lock().unwrap().len();
                println!("[Time: {}] Number of clients connected = {}", format_time(), count);
                while running.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_secs(10));
                    let count = clients.lock().unwrap().len();
                    println!("[Time: {}] Number of clients connected = {}", format_time(), count);
                }
            });
        }

        for stream in listener.incoming() {
            if !running.load(Ordering::SeqCst) {
                break;
            }
            let mut stream = stream?;
            let request_count = Arc::clone(&request_count);
            let clients = Arc::clone(&clients);
            let client_id_counter = Arc::clone(&client_id_counter);
            let running = Arc::clone(&running);
            let client_addr = stream.peer_addr().unwrap();

            let client_id = {
                let mut id_lock = client_id_counter.lock().unwrap();
                let id = *id_lock;
                *id_lock += 1;
                id
            };

            {
                let mut clients_lock = clients.lock().unwrap();
                clients_lock.insert(client_id, stream.try_clone().unwrap());
                println!("[Time: {}] Client {} connected. Number of clients connected = {}",
                    format_time(), client_id, clients_lock.len());
            }

            thread::spawn(move || {
                let mut buffer = [0; 512];
                loop {
                    if !running.load(Ordering::SeqCst) {
                        break;
                    }
                    let size = match stream.read(&mut buffer) {
                        Ok(0) => {
                            let mut clients_lock = clients.lock().unwrap();
                            clients_lock.remove(&client_id);
                            println!("[Time: {}] Client {} disconnected. Number of clients connected = {}",
                                format_time(), client_id, clients_lock.len());
                            break;
                        }
                        Ok(n) => n,
                        Err(_e) => {
                            let mut clients_lock = clients.lock().unwrap();
                            clients_lock.remove(&client_id);
                            println!("[Time: {}] Client {} disconnected. Number of clients connected = {}",
                                format_time(), client_id, clients_lock.len());
                            break;
                        }
                    };

                    let msg = from_utf8(&buffer[..size]).unwrap_or("");
                    println!("Received: {}", msg);
                    {
                        let mut count = request_count.lock().unwrap();
                        *count += 1;
                    }



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
                        format!("client IP = {}, port = {}", client_addr.ip(), client_addr.port())
                    } else if msg == "OPT4" {
                        let count = request_count.lock().unwrap();
                        format!("requests served = {}", *count)
                    } else if msg == "OPT5"{
                        let mut clients_lock = clients.lock().unwrap();
                        clients_lock.remove(&client_id);
                        println!("[Time: {}] Client {} disconnected. Number of clients connected = {}",
                            format_time(), client_id, clients_lock.len());
                        break;
                    } 
                    else {
                        "Invalid message.".to_string()
                    };
                    
                    stream.write_all(reply.as_bytes()).unwrap();
                }
            });
        }
        Ok(())
    }
