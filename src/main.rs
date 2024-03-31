use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::{thread, time};
use regex::Regex;
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

fn main() {
    let local_ip = "127.0.0.1:".to_string();

    println!("Enter your username: ");
    let user = get_input().trim().to_string();

    println!("Enter your port: ");
    let port = get_input().trim().to_string();

    let address = local_ip.clone() + &port;

    println!("Address: {}", address);

    let mut contacts: Vec<Contact> = Vec::new();
    let inbox = Arc::new(Mutex::new(Vec::<Message>::new()));

    handle_receiving(address.clone(), Arc::clone(&inbox));

    loop {
        match handle_user_input(&user, &mut contacts, &inbox) {
            Ok(should_exit) => {
                if should_exit {
                    break;
                }
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    println!("Program ended");
}

fn handle_user_input(user: &str, mut contacts: &mut Vec<Contact>, inbox: &Arc<Mutex<Vec<Message>>>) -> Result<bool, String> {
    let input = get_input().trim().to_string();
    let separated_by_quotes: Vec<&str> = input.split('"').collect();
    let separated_by_spaces: Vec<&str> = separated_by_quotes.get(0).unwrap_or(&"").split_whitespace().collect();
    let first_element = separated_by_spaces.get(0).unwrap_or(&"");

    match *first_element {
        "send" => {
            send_message_command(&user, &separated_by_spaces, &separated_by_quotes, contacts)?;
        }
        "add" => {
            add_contact_command(&mut contacts, &separated_by_spaces)?;
        }
        "print" => {
            print_command(&separated_by_spaces, &contacts)?;
        }
        "help" => {
            print_help();
        }
        "quit" => {
            return Ok(true);
        }
        "listen" => {
            listen_command(inbox);
        }
        _ => {
            println!("Input invalid");
        }
    }

    Ok(false)
}

fn send_message_command(user: &str, separated_by_spaces: &[&str], separated_by_quotes: &[&str], contacts: &Vec<Contact>) -> Result<(), String> {
    let recipient = separated_by_spaces.get(1).ok_or("Recipient not specified")?;
    let message = separated_by_quotes.get(1).ok_or("Message not specified")?;
    if is_ip_addr(recipient) {
        send_message(user.to_string(), recipient, message);
    } else {
        let mut found_contact = false;
        for contact in contacts.iter() {
            if recipient == &contact.name {
                send_message(user.to_string(), &contact.address, message);
                found_contact = true;
                break;
            }
        }
        if !found_contact {
            println!("Invalid IP or contact name");
        }
    }
    Ok(())
}

fn add_contact_command(contacts: &mut Vec<Contact>, separated_by_spaces: &[&str]) -> Result<(), String> {
    let name = separated_by_spaces.get(1).ok_or("Name not specified")?.to_string();
    let address = separated_by_spaces.get(2).ok_or("Address not specified")?.to_string();
    contacts.push(Contact { name, address });
    Ok(())
}

fn print_command(separated_by_spaces: &[&str], contacts: &Vec<Contact>) -> Result<(), String> {
    match separated_by_spaces.get(1) {
        Some(&"contacts") => {
            print_contacts(&contacts);
        }
        _ => {
            println!("Invalid print command");
        }
    }
    Ok(())
}

fn listen_command(inbox: &Arc<Mutex<Vec<Message>>>) {
    let should_exit = Arc::new(Mutex::new(false));
    let thread_should_exit = Arc::clone(&should_exit);
    let cloned_inbox = Arc::clone(inbox);

    thread::spawn(move || {
        loop {
            if *thread_should_exit.lock().unwrap() {
                break;
            }
            check_inbox(Arc::clone(&cloned_inbox));
            thread::sleep(time::Duration::from_secs(1));
        }
    });

    let _ = get_input();
    *should_exit.lock().unwrap() = true;
}

fn is_ip_addr(str: &str) -> bool {
    let re = Regex::new(r"^\d+\.\d+\.\d+\.\d+:\d+$").unwrap();
    re.is_match(str)
}

fn port_is_open(destination_address: String, timeout: u64) -> bool {
    let addr: SocketAddr = destination_address.parse().expect("Invalid address");

    match TcpStream::connect_timeout(&addr, time::Duration::from_secs(timeout)) {
        Ok(_) => {
            println!("Port is open");
            true
        }
        Err(_) => {
            println!("Port is closed");
            false
        }
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

fn print_contacts(contacts: &Vec<Contact>) {
    println!("Contacts:");
    if contacts.is_empty() {
        println!("No contacts saved.");
    } else {
        for (i, contact) in contacts.iter().enumerate() {
            println!("{}. {} - {}", i, contact.name, contact.address);
        }
    }
}

fn send_message(username: String, destination_address: &str, message: &str) {
    if !port_is_open(destination_address.to_string(), 1) {
        println!("Recipient is offline!");
        return;
    }

    let (sender, receiver) = channel::<String>();
    sender.send(username.clone() + "%%" + message).unwrap();
    let message = match receiver.recv() {
        Ok(msg) => msg,
        Err(_) => return,
    };

    if let Ok(mut stream) = TcpStream::connect(destination_address) {
        if let Err(e) = stream.write_all(message.as_bytes()) {
            eprintln!("Error sending message: {}", e);
        }
    } else {
        eprintln!("Failed to connect to server at {}", destination_address);
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
                        if received_message.trim().is_empty() {
                            continue;
                        }
                        let contents: Vec<&str> = received_message.split("%%").collect();
                        let new_message = Message {
                            sender: contents.get(0).unwrap_or(&"").to_string(),
                            message: contents.get(1).unwrap_or(&"").to_string(),
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
