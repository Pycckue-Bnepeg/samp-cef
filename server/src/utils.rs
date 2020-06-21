use log::error;
use std::collections::VecDeque;

pub struct IdPool {
    pool: VecDeque<u32>,
}

impl IdPool {
    pub fn new(capacity: usize) -> IdPool {
        let mut vec = VecDeque::with_capacity(capacity);

        for i in 0..capacity {
            vec.push_back(i as u32);
        }

        IdPool { pool: vec }
    }

    pub fn get(&mut self) -> Option<u32> {
        self.pool.pop_front()
    }

    pub fn reset(&mut self, value: u32) {
        self.pool.push_back(value);
    }
}

pub fn handle_result<T, E: std::fmt::Debug>(result: Result<T, E>) -> Option<T> {
    if let Err(err) = result.as_ref() {
        error!("{:?}", err);
    }

    result.ok()
}
