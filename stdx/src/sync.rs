use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

use crate::{Logger, error::Error};

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub struct WorkerPool {
    logger: Logger,
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl WorkerPool {
    pub fn build(logger: Logger, size: usize) -> Result<WorkerPool, Error> {
        if size == 0 {
            return Err(Error::from("N"));
        }

        Ok(Self::new(logger, size))
    }

    fn new(logger: Logger, size: usize) -> Self {
        let mut workers = Vec::with_capacity(size);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..size {
            workers.push(Worker::new(id, logger.clone(), Arc::clone(&receiver)));
        }

        WorkerPool {
            logger,
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, _callback: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(_callback);
        let _ = self.sender.as_ref().unwrap().send(job).inspect_err(|err| {
            self.logger
                .log(&format!("Error sending job to worker pool: {}", err))
        });
    }
}

impl Clone for WorkerPool {
    fn clone(&self) -> Self {
        Self::new(self.logger.clone(), self.workers.len())
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            self.logger
                .log(&format!("Shutting down worker {}", worker.id));

            worker.thread.take().unwrap().join().unwrap();
        }
    }
}

#[derive(Debug)]
pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: usize, logger: Logger, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let logger = logger.with(&format!("worker-{}", id));

        let thread = Some(thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        logger.log(&format!("Received job from worker pool"));
                        job();
                        logger.log(&format!("Job completed"));
                    }
                    Err(_) => {
                        logger.log(&format!("Worker pool closed"));
                        break;
                    }
                }
            }
        }));

        Worker { id, thread }
    }
}
