use std::{
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8080";

fn main() {
    println!("[crokd] application version {}", env!("CARGO_PKG_VERSION"));

    let addr = Config::get_default("CROKD_HTTP_ADDR", DEFAULT_HTTP_ADDR);
    let listener = TcpListener::bind(&addr).expect("Failed to bind to address");

    println!("[crokd] listen address {} ...", addr);

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

#[derive(Debug)]
struct Config;

impl Config {
    pub fn get(key: &str) -> Option<String> {
        let value = env::var(key);
        match value {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    pub fn get_default(key: &str, default: &str) -> String {
        match Self::get(key) {
            Some(value) => value,
            None => default.to_string(),
        }
    }
}

#[derive(Debug)]
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
            "[crokd] incoming request from {} {{bytes={}}}",
            remote_addr,
            request.len()
        );

        let response = format!("HTTP/1.1 200 OK\r\n\r\nHello, World!");
        stream.write_all(response.as_bytes()).unwrap();
    }
}
