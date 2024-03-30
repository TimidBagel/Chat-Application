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
        let separated_by_quotes: Vec<&str> = input.split("\"").collect();
        let separated_by_spaces: Vec<&str> = separated_by_quotes.get(0).expect("element not found").split(" ").collect();
        let first_element: &str = separated_by_spaces.get(0).expect("element not found");

        if first_element == "send" {
            let recipient: &str = separated_by_spaces.get(1).expect("element not found");
            let message: &str = separated_by_quotes.get(1).expect("element not found");
            if is_ip_addr(recipient) {
                send_message(user.clone(), recipient, message);
                continue;
            }

            let mut found_contact: bool = false;        
            for contact in contacts.clone() {
                if recipient.to_string() == contact.name {
                    send_message(user.clone(), &contact.address, message);
                    found_contact = true;
                }
            }

            if !found_contact {println!("invalid IP or contact name");}
            continue;
        } 
        else if first_element == "add" {
            let name = separated_by_spaces.get(1).expect("element not found");
            let address = separated_by_spaces.get(2).expect("element not found");
            contacts = add_contact(contacts.clone(), &name, &address);
        } 
        else if first_element == "print" {
            let second_element: &str = separated_by_spaces.get(1).expect("element not found");
            if second_element == "contacts" {
                print_contacts(contacts.clone());
            }
        }
        else if first_element == "help" {
            print_help();
        }
        else if first_element == "quit" {
            break;
        } 
        else {
            // invalid input
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
    println!("
print contacts                     - prints list of saved contacts
send [address or name] \"[message]\" - sends a message to a specified recipient
add [name] [address]               - adds a new contact with specified name and IP address
quit                               - ends program promptly");
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

fn add_contact(mut contacts: Vec<Contact>, new_name: &str, new_address: &str) -> Vec<Contact> {
    let mut new_contact: Vec<Contact> = vec![Contact {
        name: new_name.to_string(),
        address: new_address.to_string(),
    }];

    contacts.append(&mut new_contact);

    return contacts
}

fn send_message(username: String, destination_address: &str, message: &str) {
    let (sender, receiver) = channel::<String>();

    sender.send(username + "%%" + message).unwrap();
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