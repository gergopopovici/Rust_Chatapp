use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc,Mutex};
use std::thread;

type ClientsMap = Arc<Mutex<HashMap<String,TcpStream>>>;

fn handle_client(mut stream:TcpStream, clients:ClientsMap){
    let mut buffer = [0;512];
    let bytes_read = stream.read(&mut buffer).unwrap();
    let mut username = String::from_utf8_lossy(&buffer[..bytes_read]).to_string().trim().to_string();
    loop{
        let mut clients_guard = clients.lock().unwrap();
        if clients_guard.contains_key(&username) {
            stream.write_all(b"Username already taken, please choose another\n").unwrap();
            let bytes_read = stream.read(&mut buffer).unwrap();
            username = String::from_utf8_lossy(&buffer[..bytes_read]).to_string().trim().to_string();
        } else {
            clients_guard.insert(username.clone(), stream.try_clone().unwrap());
            break;
        }
    }
    stream.write_all(b"Welcome to the chat!\n").unwrap();
    loop{
        let bytes_read = stream.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut parts = message.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let rest = parts.collect::<Vec<&str>>().join(" ");
        match cmd{
            "/logout"=>{
                clients.lock().unwrap().remove(&username);
                break;
            }
            "/list" =>{
                let client_guard = clients.lock().unwrap();
                let list = client_guard.keys().cloned().collect::<Vec<String>>().join(",");
                let response = format!("Active users: {}\n", list);
                if let Err(e) = stream.write_all(response.as_bytes()){
                    eprintln!("Failed to send list to {}: {}", username, e);
                }
            }
            "/pm" =>{
                let mut parts = rest.splitn(2," ");
                let recipient = parts.next().unwrap_or("");
                let msg = parts.next().unwrap_or("");
                let client_guard = clients.lock().unwrap();
                if let Some(mut recipient_stream) = client_guard.get(recipient){
                    recipient_stream.write_all(format!("(Private message) from {}: {}\n",username,msg).as_bytes()).unwrap();
                }else{
                    stream.write_all(b"Recipient not found\n").unwrap();
                }
            }
            _=>{
                let client_guard = clients.lock().unwrap();
                for (client_name, mut client) in client_guard.iter(){
                    if client_name != &username {
                        client.write_all(format!("{}: {}\n", username, message).as_bytes()).unwrap();
                    }
                }
               }
       }
    }
}
fn main(){
    let server = TcpListener::bind("localhost:8080").expect("Failed to bind to port 8080");
    let clients: ClientsMap = Arc::new(Mutex::new(HashMap::new()));
    println!("Server listening on port 8080");
    for stream in server.incoming(){
        match stream{
            Ok(stream) =>{
                let clients = Arc::clone(&clients);
                thread::spawn(move || {
                    handle_client(stream, clients);
                });
            }
            Err(e)=>{
                eprintln!("Failed to establish a connection: {}", e);
            }
        }
    }
}