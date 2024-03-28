use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::{result, thread};
use regex::Regex;
use std::time::Duration;

#[derive(Clone, Debug)]
struct Contact {
    name: String,
    address: String,
}

/*
    TO DO: 
    - function for live chat
    - more robust error handling
    - message protocol
    - better command input system
    - handling typing and message receiving overlap
 */

// message protocol: username%%message

fn main() {
    let local_ip = "127.0.0.1:".to_string();

    println!("Enter your username: ");
    let user = get_input().trim().to_string();

    println!("Enter your port: ");
    let binding = get_input();
    let port = binding.trim();

    let address: String = local_ip.clone() + port;

    println!("address: {}", address);

    let mut contacts: Vec<Contact> = vec![];
    
    handle_receiving(address.clone());

    loop {
        let input = get_input().trim().to_string();
        if input == "help".to_string() {
            print_help();
        } 
        else if input == "display contacts".to_string() {
            print_contacts(contacts.clone());
        } 
        else if input == "add contact".to_string() {
            contacts = add_contact(contacts);
        } 
        else if input == "send message".to_string() {
            select_contact_for_chat(user.clone(), contacts.clone());
        } 
        else if input == "quit".to_string() {
            break;
        }
        else {
            println!("'{}' not recognized as internal command.", input)
        }
    }

    println!("program ended");
}

fn is_ip_addr(str: &str) -> bool{
    let re = Regex::new(r"^\d+\.\d+\.\d+\.\d+:\d+$").unwrap();

    if re.is_match(str) {
        true
    } else {
        false
    }
}

fn port_is_open(destination_address: String, timeout: u64) -> bool {
    let addr: SocketAddr = destination_address.parse().expect("Invalid address");

    match TcpStream::connect_timeout(&addr, Duration::from_secs(timeout)) {
        Ok(_) => {println!("Port is open"); true},
        Err(_) => {println!("Port is closed"); false},
    }
}

fn get_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input
}

fn print_help() {
    println!("display contacts - prints saved contacts
add contact      - starts add contact process
send message     - send a message to contact
quit             - quits program");
}

fn print_contacts(contacts: Vec<Contact>) {
    println!("Contacts:");

    if contacts.len() < 1 {
        println!("No contacts saved.");
    } 

    else {
        let i = 0;
        for contact in contacts.clone() {
            println!("{}. {} - {}", i, contact.name, contact.address);
        }
    }
}

fn add_contact(mut contacts: Vec<Contact>) -> Vec<Contact> {
    println!("Enter new contact name: ");
    let new_name = get_input().trim().to_string();

    println!("Enter {}'s IP address: ", new_name);
    let new_address = get_input().trim().to_string();

    let mut new_contact: Vec<Contact> = vec![Contact {
        name: new_name,
        address: new_address,
    }];

    contacts.append(&mut new_contact);

    return contacts
}

fn select_contact_for_chat(username: String, contacts: Vec<Contact>) {
    println!("Enter contact name or IP to send a message or enter '_back':");
    let input = get_input().trim().to_string();

    if input == "_back".to_string() {
        return
    }

    if is_ip_addr(&input) {
        send_message(username, input);
        return;
    }

    for contact in contacts.clone() {
        if input == contact.name {
            send_message(username, contact.address);
            return
        }
    }

    println!("Input is not valid contact name or IP.");
    select_contact_for_chat(username, contacts);
}

fn send_message(username: String, destination_address: String) {
    let (sender, receiver) = channel::<String>();

    let input = username.to_owned() + "%%" + get_input().trim();
    sender.send(input.trim().to_string()).unwrap();
    let message = match receiver.recv() {
        Ok(msg) => msg,
        Err(_) => return,
    };

    if let Ok(mut stream) = TcpStream::connect(&destination_address) {
        if let Err(e) = stream.write_all(message.as_bytes()) {
            eprintln!("Error sending message: {}", e);
        }
    } else {
        eprintln!("Failed to connect to sevrer at {}", destination_address);
    }
}

fn handle_receiving(address: String) {
    thread::spawn(move || {
        let listener = TcpListener::bind(&address).expect("Failed to bind to address");
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0; 1024];
                    if let Ok(bytes_read) = stream.read(&mut buffer) {
                        let received_message = String::from_utf8_lossy(&buffer[..bytes_read]);

                        println!("raw message: '{}'", received_message);

                        if received_message.trim() == "" {
                            println!("Received ping message");
                            continue;
                        }

                        let contents: Vec<&str> = received_message.split("%%").collect();

                        let sender = contents.get(0).expect("element not found").to_string();
                        
                        println!("Received message from {}: {}", sender, contents.get(1).expect("element not found").to_string());
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    });
}