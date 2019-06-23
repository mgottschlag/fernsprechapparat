//! Type which generates events when the earpiece is picked up or dropped.

use super::gpio::InputPin;
use super::Event;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Interface to the earpiece hook switch.
///
/// The implementation assumes that the switch is active-low. If the GPIO value
/// is `false`, the implementation assumes that the earpiece is on the hook,
/// whereas `true` signals that the earpiece has been picked up.
pub struct Earpiece {
    thread: Option<JoinHandle<()>>,
    stop_thread: Arc<AtomicBool>,
}

impl Earpiece {
    pub fn new<Pin: InputPin + Send + 'static>(hook: Pin, sender: Sender<Event>) -> Self {
        let stop_thread = Arc::new(AtomicBool::new(false));
        let stop_copy = stop_thread.clone();
        let thread = thread::spawn(move || {
            let mut picked_up = false;
            loop {
                // Wait with timeout to allow Drop to terminate the thread in a
                // timely fashion.
                let wait_result = hook.wait_timeout(Duration::from_millis(1000));
                if stop_thread.load(Ordering::SeqCst) {
                    return;
                }
                if wait_result {
                    let pin_state = hook.read();
                    if pin_state != picked_up {
                        let result = sender.send(if pin_state {
                            Event::EarpiecePickedUp
                        } else {
                            Event::EarpiecePutDown
                        });
                        if result.is_err() {
                            break;
                        }
                        picked_up = pin_state;
                    }
                }
            }
        });
        Self {
            thread: Some(thread),
            stop_thread: stop_copy,
        }
    }
}

impl Drop for Earpiece {
    fn drop(&mut self) {
        self.stop_thread.store(true, Ordering::SeqCst);
        let thread = self.thread.take();
        thread.unwrap().join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpio::sim::{SimEnvironment, SimInputPin};

    use std::sync::mpsc::{channel, Receiver};
    use std::thread::sleep;

    #[test]
    fn test_earpiece() {
        const HOOK_PIN: usize = 0;

        let env = SimEnvironment::new();
        let hook = env.create_input_pin(HOOK_PIN, false);
        env.write_input(HOOK_PIN, false);

        let (send, recv) = channel();
        let earpiece = Earpiece::new::<SimInputPin>(hook, send);

        // Make sure the thread is ready.
        sleep(Duration::from_millis(10));
        assert!(recv.try_recv().is_err());

        env.write_input(HOOK_PIN, true);
        sleep(Duration::from_millis(10));
        assert_eq!(recv.try_recv(), Ok(Event::EarpiecePickedUp));

        env.write_input(HOOK_PIN, false);
        sleep(Duration::from_millis(10));
        assert_eq!(recv.try_recv(), Ok(Event::EarpiecePutDown));
    }
}
