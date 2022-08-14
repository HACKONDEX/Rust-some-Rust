#![forbid(unsafe_code)]

use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    thread,
};

////////////////////////////////////////////////////////////////////////////////

pub struct ThreadPool {
    thread_count: usize,
    workers: Vec<thread::JoinHandle<()>>,
    task_queue: Sender<Box<dyn FnOnce() + 'static + Send>>,
    shutdown_notifier: Sender<i32>,
    shutdown_receiver: Receiver<i32>,
}

impl ThreadPool {
    pub fn new(thread_count: usize) -> Self {
        let (shutdown_notifier, thread_receiver) = unbounded::<i32>();
        let (finish_sender, shutdown_receiver) = unbounded::<i32>();
        let (task_queue, task_receiver) = unbounded();
        let mut workers = Vec::new();
        for _i in 0..thread_count {
            let shutdown_receiver = thread_receiver.clone();
            let pthread_task_receiver = task_receiver.clone();
            let pthread_finisher = finish_sender.clone();
            workers.push(thread::spawn(move || {
                while shutdown_receiver.is_empty() || !pthread_task_receiver.is_empty() {
                    let task = pthread_task_receiver.recv().unwrap();
                    // Safe Execute
                    catch_unwind(AssertUnwindSafe(task)).ok();
                }
                pthread_finisher.send(0).ok();
            }));
        }
        Self {
            thread_count,
            workers,
            task_queue,
            shutdown_notifier,
            shutdown_receiver,
        }
    }

    pub fn spawn<F, T>(&self, task: F) -> JoinHandle<T>
    where
        F: FnOnce() -> T + 'static + Send,
        T: Send + 'static,
    {
        let (result_sender, result_receiver) = unbounded::<T>();
        self.task_queue
            .send(Box::new(move || {
                result_sender.send(task()).ok();
            }))
            .ok();
        JoinHandle::new(result_receiver)
    }

    pub fn shutdown(self) {
        for _i in 0..self.thread_count {
            self.shutdown_notifier.send(42).ok();
        }
        for _i in 0..self.thread_count {
            self.shutdown_receiver.recv().ok();
        }
        for x in self.workers {
            x.join().ok();
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct JoinHandle<T> {
    result_receiver: Receiver<T>,
}

#[derive(Debug)]
pub struct JoinError {}

impl<T: Send> JoinHandle<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        Self {
            result_receiver: receiver,
        }
    }
    pub fn join(self) -> Result<T, JoinError> {
        let result = self.result_receiver.recv();
        if let Ok(x) = result {
            return Ok(x);
        }
        Err(JoinError {})
    }
}
