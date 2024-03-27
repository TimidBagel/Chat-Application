use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;


fn main() {
    let local_ip = "127.0.0.1:".to_string();
    println!("Enter your username: ");
    let _user = get_input().trim().to_string();
    println!("Enter your port: ");
    let port = get_input().trim().to_string();
    let address: String = local_ip.clone() + &port;

    println!("address: {}", address);

    let (sender, receiver) = channel::<String>();

    let listener = TcpListener::bind(&address).expect("Failed to bind to address");

    println!("Enter the destination port: ");
    let dest_port = get_input().trim().to_string();
    let target_address: String = local_ip.clone() + &dest_port;

    handle_sending(target_address, sender.clone(), receiver);
    handle_receiving(listener);
}

fn get_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input
}

fn handle_sending(destination_address: String, sender: Sender<String>, receiver: Receiver<String>) {
    thread::spawn(move || {
        loop {
            let message = match receiver.recv() {
                Ok(msg) => msg,
                Err(_) => break,
            };

            if let Ok(mut stream) = TcpStream::connect(&destination_address) {
                if let Err(e) = stream.write_all(message.as_bytes()) {
                    eprintln!("Error sending message: {}", e);
                }
            } else {
                eprintln!("Failed to connect to sevrer at {}", destination_address);
            }
        }
    });

    thread::spawn(move || {
        loop {
            let input = get_input();
            sender.send(input.trim().to_string()).unwrap();
        }
    });
}

fn handle_receiving(listener: TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {

                // Spawn a thread to handle the client's message
                thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        if let Ok(bytes_read) = stream.read(&mut buffer) {
                            let received_message = String::from_utf8_lossy(&buffer[..bytes_read]);
                            println!("Received message from other: {}", received_message);
                        }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}