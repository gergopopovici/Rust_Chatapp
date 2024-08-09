use std::io::{self, Read, Write, BufRead};
use std::net::TcpStream;
use std::thread;

fn main() {
    let mut client = TcpStream::connect("localhost:8080")
        .expect("Failed to connect to server");
    println!("What's your username?");
    let mut username = String::new();
    io::stdin().read_line(&mut username)
        .expect("Failed to read username");
    client.write_all(username.as_bytes())
        .expect("Failed to send username");

    let mut client_clone_send = client.try_clone()
        .expect("Failed to clone client");

    thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let message = line.expect("Failed to read line");
            client_clone_send.write_all(message.as_bytes())
                .expect("Failed to send message");
        }
    });

    let mut buffer = [0; 512];
    loop {
        let bytes_read = client.read(&mut buffer)
            .expect("Failed to read from server");
        if bytes_read == 0 {
            println!("Connection closed by server.");
            break;
        }
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        print!("{}", message);
    }
}
