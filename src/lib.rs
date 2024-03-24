use core::fmt;
use std::{
    sync::{mpsc, Arc, Mutex}, 
    thread
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>
}

impl ThreadPool {
    /// Create a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool.
    /// 
    /// # Panics
    /// 
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let workers = Self::create_workers(size, receiver);
        
        ThreadPool { 
            workers, 
            sender: Some(sender),
        }
    }
    
    /// Builds a new ThreadPool.
    /// 
    /// The size is the number of threads in the pool.
    /// 
    /// # Errors
    /// 
    /// Returns a `PoolCreationError` if size is 0
    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            return Err(PoolCreationError::from("Size should be greater than 0"));
        }
        
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        // let workers = Self::create_workers(size);
        let workers = Self::create_workers(size, receiver);

        Ok(ThreadPool { 
            workers, 
            sender: Some(sender),
        })
    }

    pub fn execute<F>(&self, f:F)
    where
        F: FnOnce() + Send + 'static 
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }

    fn create_workers(size: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Vec<Worker> {
        let mut workers = Vec::with_capacity(size);
            
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
    
        workers
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

#[derive(Debug)]
pub struct PoolCreationError {
    message: String
}

impl fmt::Display for PoolCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error creating ThreadPool: {}", self.message)
    }
}

impl PoolCreationError {
    pub fn from(message: &str) -> PoolCreationError {
        PoolCreationError{
            message: message.to_string()
        }
    }
}


struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    
                    job();
                },
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down");
                    break;
                },
            }
        });

        Worker { 
            id, 
            thread: Some(thread),
        }
    }
}