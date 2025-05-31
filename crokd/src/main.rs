use std::{
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use stdx::{Error, Logger};

const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8080";

fn main() {
    let logger = Logger::default().with("crokd");

    logger.log("Starting application ...");
    logger.log(&format!(
        "Application version {}",
        env!("CARGO_PKG_VERSION")
    ));

    let addr = Config::get_default("CROKD_HTTP_ADDR", DEFAULT_HTTP_ADDR);
    let tcp_srv = TcpServer::new(logger.clone(), Handler::new(logger.clone()));

    match tcp_srv.listen(&addr) {
        Ok(_) => {}
        Err(err) => {
            logger.log(&format!("Failed to start listen: {}", err));
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
struct TcpServer {
    logger: Logger,
    handler: Handler,
}

impl TcpServer {
    fn new(logger: Logger, handler: Handler) -> Self {
        TcpServer { logger, handler }
    }

    fn listen(&self, addr: &str) -> Result<(), Error> {
        let listener =
            TcpListener::bind(&addr).map_err(|err| Error::from(err).wrap("bind address"))?;

        self.logger.log(&format!("Listen address {} ...", addr));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.handler.execute(stream);
                }
                Err(err) => {
                    self.logger
                        .log(&format!("Failed to accept connection: {}", err));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Handler {
    logger: Logger,
}

impl Handler {
    fn new(logger: Logger) -> Self {
        Handler { logger }
    }

    fn execute(&self, mut stream: TcpStream) {
        let remote_addr = stream.peer_addr().unwrap();

        let reader = BufReader::new(&stream);

        let request: Vec<String> = reader
            .lines()
            .map(|line| line.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        self.logger.log(&format!(
            "Incoming request from {} {{bytes={}}}",
            remote_addr,
            request.len()
        ));

        let response = format!("HTTP/1.1 200 OK\r\n\r\nHello, World!");
        stream.write_all(response.as_bytes()).unwrap();
    }
}
