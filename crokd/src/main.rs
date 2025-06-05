use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use stdx::{Error, Logger, env, log::Level as LogLevel, sync::WorkerPool};

const DEFAULT_HTTP_ADDR: &str = "127.0.0.1:8080";
const DEFAULT_WORKER_POOL_SIZE: usize = 10;

#[derive(Debug)]
struct Config {
    pub version: String,
    pub http_addr: String,
    pub worker_pool_size: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, Error> {
        let mut cfg = Self::default();

        cfg.version = env!("CARGO_PKG_VERSION").to_string();

        cfg.http_addr =
            env::get_with_default("CROKD_HTTP_ADDR", DEFAULT_HTTP_ADDR.to_string().as_str())
                .to_string();

        cfg.worker_pool_size = env::get_with_default(
            "CROKD_WORKER_POOL_SIZE",
            DEFAULT_WORKER_POOL_SIZE.to_string().as_str(),
        )
        .parse::<usize>()
        .map_err(|err| Error::from(err.to_string()).wrap("invalid worker pool size"))?;

        Ok(cfg)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: "0.0.1".to_string(),
            http_addr: DEFAULT_HTTP_ADDR.to_string(),
            worker_pool_size: DEFAULT_WORKER_POOL_SIZE,
        }
    }
}

fn main() {
    let logger = Logger::default()
        .with_constraint(LogLevel::Debug)
        .with_level(LogLevel::Debug)
        .with("crokd");

    let sys_logger = logger
        .with_constraint(LogLevel::System)
        .with_level(LogLevel::System);

    let cfg = Config::from_env()
        .inspect(|_| logger.log("Inintializing config ..."))
        .inspect_err(|err| {
            sys_logger
                .with_level(LogLevel::SystemError)
                .log(&format!("Failed to load config: {}", err))
        })
        .unwrap();

    logger.log(&format!("Application version {}", cfg.version));
    logger.log("Starting application ...");

    let worker_pool =
        WorkerPool::build(sys_logger.with("worker-pool").noop(), cfg.worker_pool_size)
            .inspect_err(|err| {
                sys_logger
                    .with_level(LogLevel::SystemError)
                    .log(&format!("Failed to init worker pool: {}", err))
            })
            .unwrap();

    let handler = Handler::new(logger.with("handler"));

    let tcp_server = TcpServer::new(sys_logger.with("tcp-server"), worker_pool, handler);

    tcp_server
        .listen(&cfg.http_addr)
        .inspect_err(|err| {
            sys_logger
                .with_level(LogLevel::SystemError)
                .log(&format!("Failed to start listen: {}", err))
        })
        .unwrap();
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
                        .with_level(LogLevel::SystemError)
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
