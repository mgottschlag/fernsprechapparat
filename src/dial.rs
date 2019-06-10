//! Interface to a rotary dial connected via GPIOs.

use super::gpio::{InputPin, InputPinGroup};
use super::Event;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Index of the nsa contact in the pin group.
const NSA: usize = 0;
/// Index of the nsi contact in the pin group.
const NSI: usize = 1;

/// Interface to a rotary dial.
///
/// See [Wikipedia](https://de.wikipedia.org/wiki/Nummernschalter) for a
/// description of pulse dialing. The names used for the contacts are taken from
/// that german description.
///
/// The implementation assumes that the switches are active-low, meaning that a
/// pin value of `false` signals that the switch has been closed.
pub struct Dial {
    thread: Option<JoinHandle<()>>,
    stop_thread: Arc<AtomicBool>,
}

impl Dial {
    pub fn new<Pin: InputPin + Send + 'static>(nsa: Pin, nsi: Pin, sender: Sender<Event>) -> Self {
        let stop_thread = Arc::new(AtomicBool::new(false));
        let stop_copy = stop_thread.clone();
        let thread = thread::spawn(move || {
            let pins = Pin::create_group(vec![Box::new(nsa), Box::new(nsi)]);

            let mut counting = false;
            let mut count = 0;
            let mut impulse = false;

            loop {
                // Wait with timeout to allow Drop to terminate the thread in a
                // timely fashion.
                let wait_result = pins.wait_timeout(Duration::from_millis(1000));
                if stop_thread.load(Ordering::SeqCst) {
                    return;
                }
                if wait_result.is_some() {
                    let pin_state = pins.read();
                    // "== 0" because the inputs are active-low
                    let nsa = (pin_state & (1 << NSA)) == 0;
                    let nsi = (pin_state & (1 << NSI)) == 0;
                    if nsa && !counting {
                        counting = true;
                        count = 0;
                    } else if !nsa && counting {
                        counting = false;
                        // 10 impulses = '0'
                        let digit = match count {
                            10 => 0,
                            count => count,
                        };
                        // 0 impulses or more than 10 impulses = invalid
                        if count != 0 && count <= 10 {
                            let result = sender.send(Event::Dialed(digit));
                            if result.is_err() {
                                break;
                            }
                        }
                    }
                    if counting && !nsi && !impulse {
                        impulse = true;
                        count += 1;
                    } else if counting && nsi && impulse {
                        impulse = false;
                    }
                }
            }
        });
        Self {
            //pins: Pin::create_group(vec![Box::new(nsa), Box::new(nsi)]),
            //sender,
            thread: Some(thread),
            stop_thread: stop_copy,
        }
    }
}

impl Drop for Dial {
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

    fn create_test_dial() -> (SimEnvironment, Dial, Receiver<Event>) {
        let env = SimEnvironment::new();

        // The NSA witch is initially open, the NSI switch is closed.
        let nsa = env.create_input_pin(NSA, false);
        env.write_input(NSA, true);
        let nsi = env.create_input_pin(NSI, false);
        env.write_input(NSI, false);

        let (send, recv) = channel();
        let dial = Dial::new::<SimInputPin>(nsa, nsi, send);

        (env, dial, recv)
    }

    fn send_impulses(
        env: &SimEnvironment,
        count: u32,
        impulse_width: Duration,
        pause_width: Duration,
    ) {
        for _ in 0..count {
            env.write_input(NSI, true);
            sleep(impulse_width);
            env.write_input(NSI, false);
            sleep(pause_width);
        }
    }

    #[test]
    fn test_dial() {
        let (env, dial, recv) = create_test_dial();

        // Make sure the thread is ready.
        sleep(Duration::from_millis(50));

        // Add some bogus impulses which shall be ignored.
        send_impulses(
            &env,
            3,
            Duration::from_millis(60),
            Duration::from_millis(40),
        );
        assert!(recv.try_recv().is_err());

        // Dial some numbers.
        for i in 0..11 {
            // Begin dialing.
            env.write_input(NSA, false);
            sleep(Duration::from_millis(50));
            send_impulses(
                &env,
                i,
                Duration::from_millis(60),
                Duration::from_millis(40),
            );
            env.write_input(NSA, true);
            sleep(Duration::from_millis(50));
            let result = recv.try_recv();
            if i == 0 || i > 10 {
                // 0 impulses or more than 10 impulses are invalid
                assert!(result.is_err());
            } else if i == 10 {
                // 10 impuses are '0'
                assert_eq!(result, Ok(Event::Dialed(0)));
            } else {
                assert_eq!(result, Ok(Event::Dialed(i)));
            }
        }

        // TODO: Test different timing?
    }
}
