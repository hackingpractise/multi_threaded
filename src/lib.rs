use std::sync::{mpsc, Arc, Mutex};
use std::thread;

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
type Job = Box<dyn FnOnce() + Send + 'static>;
impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let mes = receiver.lock().unwrap().recv();
            match mes {
                Ok(job) => {
                    job();
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    break;
                }
            }
        });

        Worker {
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size should be more than zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        //        let size = 2;
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            workers.push(Worker::new(receiver.clone()));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            //            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
