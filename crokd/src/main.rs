use std::{
    env,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use stdx::{Error, Logger, sync::WorkerPool};

const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_WORKER_POOL_SIZE: usize = 10;

fn main() {
    let logger = Logger::default().with("crokd");

    logger.log("Starting application ...");
    logger.log(&format!(
        "Application version {}",
        env!("CARGO_PKG_VERSION")
    ));

    let addr = Config::get_default("CROKD_HTTP_ADDR", DEFAULT_HTTP_ADDR);

    let worker_pool = WorkerPool::build(logger.with("worker-pool"), DEFAULT_WORKER_POOL_SIZE)
        .inspect_err(|err| logger.log(&format!("Failed to init worker pool: {}", err)))
        .unwrap();

    let handler = Handler::new(logger.with("handler"));

    let tcp_server = TcpServer::new(logger.with("tcp-server"), worker_pool, handler);

    tcp_server
        .listen(&addr)
        .inspect_err(|err| logger.log(&format!("Failed to start listen: {}", err)))
        .unwrap();
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
    worker_pool: WorkerPool,
    handler: Handler,
}

impl TcpServer {
    fn new(logger: Logger, worker_pool: WorkerPool, handler: Handler) -> Self {
        TcpServer {
            logger,
            worker_pool,
            handler,
        }
    }

    fn listen(&self, addr: &str) -> Result<(), Error> {
        let listener =
            TcpListener::bind(&addr).map_err(|err| Error::from(err).wrap("bind address"))?;

        self.logger.log(&format!("Listen address {} ...", addr));

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = self.handler.clone();
                    self.worker_pool.execute(move || {
                        handler.execute(stream);
                    });
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

#[derive(Debug, Clone)]
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

        thread::sleep(Duration::from_secs(10));

        let response = format!("HTTP/1.1 200 OK\r\n\r\nHello, World!");
        stream.write_all(response.as_bytes()).unwrap();
    }
}
