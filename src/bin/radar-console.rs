use std::io::{self, Read, Write};
use std::net::TcpStream;

fn main() {
    println!("Connected to localhost:7878.");
    println!("Enter commands to send to the server (type 'exit' to quit).");

    match TcpStream::connect("127.0.0.1:7878") {
        Ok(mut stream) => loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut command = String::new();
            io::stdin().read_line(&mut command).unwrap();
            let command = command.trim();

            if command.eq_ignore_ascii_case("exit") {
                break;
            }

            let command_bytes = format!("{}\r", command).into_bytes();
            if let Err(e) = stream.write_all(&command_bytes) {
                eprintln!("Failed to send command: {}", e);
                break;
            }

            let mut buffer = [0; 1024];
            match stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Server closed the connection.");
                    break;
                }
                Ok(bytes_read) => {
                    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
                    println!("Server reply: {}", response);
                }
                Err(e) => {
                    eprintln!("Failed to read from server: {}", e);
                    break;
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
        }
    }
}
