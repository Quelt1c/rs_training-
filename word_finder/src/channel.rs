use parking_lot::{Condvar, Mutex};
use std::collections::VecDeque;
use std::sync::Arc;
#[derive(Debug)]
pub struct SendError<T>(pub T);

#[derive(Debug)]
pub struct RecvError;

struct ChannelState<T> {
    queue: VecDeque<T>,
    sender_count: usize,
    receiver_count: usize,
}

struct Shared<T> {
    state: Mutex<ChannelState<T>>,
    condvar: Condvar,
}

pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Sender<T> {
    pub fn send(&self, msg: T) -> Result<(), SendError<T>> {
        let mut state = self.shared.state.lock();

        if state.receiver_count == 0 {
            return Err(SendError(msg));
        }

        state.queue.push_back(msg);

        self.shared.condvar.notify_one();
        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        let mut state = self.shared.state.lock();
        state.sender_count += 1;
        Sender {
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        let mut state = self.shared.state.lock();
        state.sender_count -= 1;

        if state.sender_count == 0 {
            self.shared.condvar.notify_all();
        }
    }
}

pub struct Receiver<T> {
    shared: Arc<Shared<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> Result<T, RecvError> {
        let mut state = self.shared.state.lock();

        while state.queue.is_empty() {
            if state.sender_count == 0 {
                return Err(RecvError);
            }

            self.shared.condvar.wait(&mut state);
        }

        Ok(state.queue.pop_front().unwrap())
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        let mut state = self.shared.state.lock();
        state.receiver_count += 1;
        Receiver {
            shared: self.shared.clone(),
        }
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        let mut state = self.shared.state.lock();
        state.receiver_count -= 1;
    }
}

pub fn unbounded<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        state: Mutex::new(ChannelState {
            queue: VecDeque::new(),
            sender_count: 1,
            receiver_count: 1,
        }),
        condvar: Condvar::new(),
    });
    (
        Sender {
            shared: shared.clone(),
        },
        Receiver { shared },
    )
}
