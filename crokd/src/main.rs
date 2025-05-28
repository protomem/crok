use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    println!("[crokd] application version {}", env!("CARGO_PKG_VERSION"));

    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");

    // Handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let handler = Handler::new();
                handler.execute(stream);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

struct Handler;

impl Handler {
    fn new() -> Self {
        Handler {}
    }

    fn execute(&self, mut stream: TcpStream) {
        let remote_addr = stream.peer_addr().unwrap();

        let reader = BufReader::new(&stream);

        let request: Vec<String> = reader
            .lines()
            .map(|line| line.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        println!(
            "Incoming request from {} {{bytes={}}}",
            remote_addr,
            request.len()
        );

        let response = format!("HTTP/1.1 200 OK\r\n\r\nHello, World!");
        stream.write_all(response.as_bytes()).unwrap();
    }
}
