#![forbid(unsafe_code)]

use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};
use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

pub enum State {
    Open,
    Closed,
}

#[derive(Error, Debug)]
#[error("channel is closed")]
pub struct SendError<T> {
    pub value: T,
}

pub struct Sender<T> {
    shared_buffer: Rc<RefCell<VecDeque<T>>>,
    shared_state: Rc<RefCell<State>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        match *(self.shared_state.borrow()) {
            State::Closed => Err(SendError { value }),
            State::Open => self.insert_new_value(value),
        }
    }

    pub fn insert_new_value(&self, value: T) -> Result<(), SendError<T>> {
        (*self.shared_buffer).borrow_mut().push_back(value);
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        match *(self.shared_state.borrow()) {
            State::Closed => true,
            State::Open => false,
        }
    }

    pub fn same_channel(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.shared_buffer, &other.shared_buffer)
    }

    pub fn new(
        shared_buffer: &Rc<RefCell<VecDeque<T>>>,
        shared_state: &Rc<RefCell<State>>,
    ) -> Self {
        Self {
            shared_buffer: Rc::clone(shared_buffer),
            shared_state: Rc::clone(shared_state),
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            shared_buffer: Rc::clone(&self.shared_buffer),
            shared_state: Rc::clone(&self.shared_state),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let links_count = Rc::strong_count(&self.shared_state);
        if links_count <= 2 {
            *(*self.shared_state).borrow_mut() = State::Closed;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("channel is empty")]
    Empty,
    #[error("channel is closed")]
    Closed,
}

pub struct Receiver<T> {
    shared_buffer: Rc<RefCell<VecDeque<T>>>,
    shared_state: Rc<RefCell<State>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Result<T, ReceiveError> {
        let mut mut_ref_buffer = (*self.shared_buffer).borrow_mut();
        if mut_ref_buffer.is_empty() {
            match *(self.shared_state.borrow()) {
                State::Closed => Err(ReceiveError::Closed),
                State::Open => Err(ReceiveError::Empty),
            }
        } else {
            Ok(mut_ref_buffer.pop_front().unwrap())
        }
    }

    pub fn close(&mut self) {
        *(*self.shared_state).borrow_mut() = State::Closed;
    }

    pub fn make_sender(&self) -> Sender<T> {
        Sender::<T>::new(&self.shared_buffer, &self.shared_state)
    }
}

impl<T> Default for Receiver<T> {
    fn default() -> Self {
        Self {
            shared_buffer: Rc::new(RefCell::new(VecDeque::<T>::new())),
            shared_state: Rc::new(RefCell::new(State::Open)),
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.close();
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let receiver = Receiver::<T>::default();
    let sender = receiver.make_sender();
    (sender, receiver)
}
