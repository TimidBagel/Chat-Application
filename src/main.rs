/*/
Note - This script is written by Iain Broomell
ChatGPT was used during the writing of this script,
though only for research purposes and redundant tasks, such as adding comments. */

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::channel;
use std::{thread, time};
use regex::Regex;
use std::sync::{Arc, Mutex};

// Define a struct representing a contact with a name and an address
#[derive(Clone, Debug)]
struct Contact {
    name: String,
    address: String,
}

// Define a struct representing a message with a sender and a message body
struct Message {
    sender: String,
    message: String,
}

fn main() {
    // Define the local IP address
    let local_ip = "127.0.0.1:".to_string();

    // Prompt the user to enter their username
    println!("Enter your username: ");
    let user = get_input().trim().to_string();

    // Prompt the user to enter the port they want to listen on
    println!("Enter your port: ");
    let port = get_input().trim().to_string();

    // Combine the local IP address and port to form the complete address
    let address = local_ip.clone() + &port;

    // Print the complete address
    println!("Address: {}", address);

    // Initialize a vector to store contacts and an Arc-wrapped Mutex to store messages
    let mut contacts: Vec<Contact> = Vec::new();
    let inbox = Arc::new(Mutex::new(Vec::<Message>::new()));

    // Start a thread to handle receiving messages
    handle_receiving(address.clone(), Arc::clone(&inbox));

    // Main loop for user interaction
    loop {
        // Handle user input
        match handle_user_input(&user, &mut contacts, &inbox) {
            Ok(should_exit) => {
                if should_exit {
                    break; // Exit the loop and end the program
                }
            }
            Err(err) => eprintln!("Error: {}", err), // Print any errors encountered
        }
    }

    println!("Program ended"); // Print a message indicating the program has ended
}

// Function to handle user input
fn handle_user_input(user: &str, mut contacts: &mut Vec<Contact>, inbox: &Arc<Mutex<Vec<Message>>>) -> Result<bool, String> {
    let input = get_input().trim().to_string(); // Read user input
    let separated_by_quotes: Vec<&str> = input.split('"').collect(); // Separate input by quotes
    let separated_by_spaces: Vec<&str> = separated_by_quotes.get(0).unwrap_or(&"").split_whitespace().collect(); // Separate input by spaces
    let first_element = separated_by_spaces.get(0).unwrap_or(&""); // Get the first element of the input

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
            return Ok(true); // Signal to exit the program
        }
        "listen" => {
            listen_command(inbox); // Start listening for incoming messages
        }
        _ => {
            println!("Input invalid"); // Print message for invalid input
        }
    }

    Ok(false) // Signal to continue the loop
}

// Function to handle the "send" command
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

// Function to handle the "add" command
fn add_contact_command(contacts: &mut Vec<Contact>, separated_by_spaces: &[&str]) -> Result<(), String> {
    let name = separated_by_spaces.get(1).ok_or("Name not specified")?.to_string();
    let address = separated_by_spaces.get(2).ok_or("Address not specified")?.to_string();
    contacts.push(Contact { name, address }); // Add a new contact to the contacts vector
    Ok(())
}

// Function to handle the "print" command
fn print_command(separated_by_spaces: &[&str], contacts: &Vec<Contact>) -> Result<(), String> {
    match separated_by_spaces.get(1) {
        Some(&"contacts") => {
            println!("Contacts:");
            if contacts.is_empty() {
                println!("No contacts saved.");
            } else {
                for (i, contact) in contacts.iter().enumerate() {
                    println!("{}. {} - {}", i, contact.name, contact.address);
                }
            }
        }
        _ => {
            println!("Invalid print command"); // Print message for invalid print command
        }
    }
    Ok(())
}

// Function to handle the "listen" command
fn listen_command(inbox: &Arc<Mutex<Vec<Message>>>) {
    let should_exit = Arc::new(Mutex::new(false));
    let thread_should_exit = Arc::clone(&should_exit);
    let cloned_inbox = Arc::clone(inbox);

    thread::spawn(move || {
        loop {
            if *thread_should_exit.lock().unwrap() {
                break; // Break out of the loop if the thread should exit
            }
            check_inbox(Arc::clone(&cloned_inbox)); // Check the inbox for new messages
            thread::sleep(time::Duration::from_secs(1)); // Sleep for 1 second
        }
    });

    let _ = get_input(); // Wait for input to exit the listening thread
    *should_exit.lock().unwrap() = true; // Set should_exit to true to signal thread to exit
}

// Function to check if a string represents an IP address
fn is_ip_addr(str: &str) -> bool {
    let re = Regex::new(r"^\d+\.\d+\.\d+\.\d+:\d+$").unwrap();
    re.is_match(str)
}

// Function to check if a port is open
fn port_is_open(destination_address: String, timeout: u64) -> bool {
    let addr: SocketAddr = destination_address.parse().expect("Invalid address");

    match TcpStream::connect_timeout(&addr, time::Duration::from_secs(timeout)) {
        Ok(_) => true,
        Err(_) => false
    }
}

// Function to get user input
fn get_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input
}

// Function to print help message
fn print_help() {
    println!("print contacts                     - prints list of saved contacts
send [address or name] \"[message]\" - sends a message to a specified recipient
add [name] [address]               - adds a new contact with specified name and IP address
quit                               - ends program promptly");
}

// Function to send a message
fn send_message(username: String, destination_address: &str, message: &str) {
    if !port_is_open(destination_address.to_string(), 1) {
        println!("Recipient is offline!"); // Print message indicating recipient is offline
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
            eprintln!("Error sending message: {}", e); // Print error message if sending message fails
        }
    } else {
        eprintln!("Failed to connect to server at {}", destination_address); // Print message if connection fails
    }
}

// Function to check the inbox for new messages
fn check_inbox(inbox: Arc<Mutex<Vec<Message>>>) {
    let mut readable_inbox = inbox.lock().unwrap();
    for message in readable_inbox.iter() {
        println!("<{}>: {}", message.sender, message.message); // Print each message in the inbox
    }
    readable_inbox.clear(); // Clear the inbox after printing messages
}

// Function to handle receiving messages
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
                            continue; // Continue to next iteration if received message is empty
                        }
                        let contents: Vec<&str> = received_message.split("%%").collect();
                        let new_message = Message {
                            sender: contents.get(0).unwrap_or(&"").to_string(),
                            message: contents.get(1).unwrap_or(&"").to_string(),
                        };
                        let mut mut_inbox = inbox.lock().unwrap();
                        mut_inbox.push(new_message); // Push the new message to the inbox
                    }
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e); // Print error message if accepting connection fails
                }
            }
        }
    });
}
