//! Main application state machine.

use super::Event;

use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

enum State {
    /// No connection to a SIP registrar.
    Unregistered,
    Ready,
    Dialing,
    IncomingCall,
    ActiveCall,
    /// We have an active call, but the connection to the SIP registrar failed.
    /// This state transitions into `Unregistered` after the call.
    ActiveCallRegistrationFailed,
    CallRejected,
}

pub struct StateMachine {
    input: Receiver<Event>,
    state: State,
}

impl StateMachine {
    pub fn new(input: Receiver<Event>) -> StateMachine {
        StateMachine {
            input,
            state: State::Ready,
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            thread::sleep(Duration::from_millis(100));
            // TODO
        }
    }
}
