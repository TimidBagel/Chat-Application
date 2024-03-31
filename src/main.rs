use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::{result, thread};
use regex::Regex;
use std::time::Duration;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
struct Contact {
    name: String,
    address: String,
}

struct Message {
    sender: String,
    message: String,
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
    let inbox = Arc::new(Mutex::new(Vec::<Message>::new()));
    
    handle_receiving(address.clone(), Arc::clone(&inbox.clone()));

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
        else if first_element == "listen" {
            let should_exit = Arc::new(Mutex::new(false));
            let thread_should_exit = Arc::clone(&should_exit);
            let cloned_inbox = Arc::clone(&inbox);

            thread::spawn(move || {
                loop {
                    if *thread_should_exit.lock().unwrap() { break; }
                    check_inbox(Arc::clone(&cloned_inbox));
                    thread::sleep(Duration::from_secs(1));
                }
            });

            let _ = get_input();
            *should_exit.lock().unwrap() = true;
        }
        else {
            println!("input invalid");
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
    println!("print contacts                     - prints list of saved contacts
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
    if !port_is_open(destination_address.to_string(), 1) {
        println!("Recipient is offline!");
    }

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

fn check_inbox(inbox: Arc<Mutex<Vec<Message>>>) {
    let mut readable_inbox = inbox.lock().unwrap();
    for message in readable_inbox.iter() {
        println!("<{}>: {}", message.sender, message.message);
    }

    readable_inbox.clear();
}

fn handle_receiving(address: String, inbox: Arc<Mutex<Vec<Message>>>) {
    thread::spawn(move || {
        let listener = TcpListener::bind(&address).expect("Failed to bind to address");
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0; 1024];
                    if let Ok(bytes_read) = stream.read(&mut buffer) {
                        let received_message = String::from_utf8_lossy(&buffer[..bytes_read]);

                        if received_message.trim() == "" {
                            continue;
                        }

                        let contents: Vec<&str> = received_message.split("%%").collect();

                        let new_message: Message = Message{
                            sender: contents.get(0).expect("element not found").to_string(),
                            message: contents.get(1).expect("element not found").to_string(),
                        };
                        
                        let mut mut_inbox = inbox.lock().unwrap();

                        mut_inbox.push(new_message);
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    });
}