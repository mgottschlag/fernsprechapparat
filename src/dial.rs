//! Interface to a rotary dial connected via GPIOs.

use super::gpio::InputPin;
use super::Event;

use std::sync::mpsc::Sender;

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
///
/// The implementation spawns a separate thread for I/O. To cleanly exit the
/// thread, close the receiving end of the mpsc channel and *then* drop the
/// `Dial` object.
pub struct Dial<Pin: InputPin> {
    pins: Pin::Group,
    sender: Sender<Event>,
}

impl<Pin: InputPin> Dial<Pin> {
    fn new(nsa: Pin, nsi: Pin, sender: Sender<Event>) -> Self {
        // TODO
        Self {
            pins: Pin::create_group(vec![Box::new(nsa), Box::new(nsi)]),
            sender,
        }
    }
}

impl<Pin: InputPin> Drop for Dial<Pin> {
    fn drop(&mut self) {
        // TODO: Stop the thread.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpio::sim::{SimEnvironment, SimInputPin};

    use std::sync::mpsc::{channel, Receiver};
    use std::thread::sleep;
    use std::time::Duration;

    fn create_test_dial() -> (SimEnvironment, Dial<SimInputPin>, Receiver<Event>) {
        let env = SimEnvironment::new();

        // The NSA witch is initially open, the NSI switch is closed.
        let nsa = env.create_input_pin(NSA, false);
        env.write_input(NSA, true);
        let nsi = env.create_input_pin(NSI, false);
        env.write_input(NSI, false);

        let (send, recv) = channel();
        let dial = Dial::new(nsa, nsi, send);

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

        drop(recv);

        // TODO: Test different timing?
    }
}
