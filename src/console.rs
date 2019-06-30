//! Alternative implementations to control the application via stdin.

use super::Event;

use std::sync::mpsc::Sender;

pub struct ConsoleInput {
    // TODO
}

impl ConsoleInput {
    pub fn new(sender: Sender<Event>) -> ConsoleInput {
        // TODO
        ConsoleInput {
            // TODO
        }
    }
}

impl Drop for ConsoleInput {
    fn drop(&mut self) {
        // TODO
    }
}
