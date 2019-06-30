//! Application which implements SIP support for modded old telephones with
//! rotary dials.
//!
//! This program is intended for a mod of a FeTAp (Fernsprechtischapparat) of
//! the Deutsche Bundespost, but will likely work with any similar phones.

extern crate pjproject;

mod console;
mod dial;
mod earpiece;
mod gpio;
mod sip;
mod state;

use console::ConsoleInput;
use dial::Dial;
use earpiece::Earpiece;
use gpio::sysfs::{SysfsInputPin, SysfsOutputPin};
use sip::Sip;
use state::StateMachine;

use std::sync::mpsc::channel;

#[derive(Debug, PartialEq)]
pub enum Event {
    Dialed(u32),
    EarpiecePickedUp,
    EarpiecePutDown,
    Registered,
    Unregistered,
}

const SIP_DOMAIN: &str = "192.168.178.1";
const SIP_USER: &str = "fernsprechapparat";
const SIP_PASSWORD: &str = "TODO";

// TODO: Correct GPIO numbers.
const NSA_PIN: usize = 1;
const NSI_PIN: usize = 2;
const RING_PIN: usize = 3;
const HOOK_PIN: usize = 4;

fn main() {
    let (input_send, input_recv) = channel();

    let sip = Sip::new(SIP_DOMAIN, SIP_USER, SIP_PASSWORD);

    let mut state_machine = StateMachine::new(input_recv);

    if cfg!(console) {
        let _input = ConsoleInput::new(input_send);
        state_machine.run();
    } else {
        let nsa = SysfsInputPin::open(NSA_PIN).unwrap();
        let nsi = SysfsInputPin::open(NSI_PIN).unwrap();
        let hook = SysfsInputPin::open(HOOK_PIN).unwrap();
        let ring = SysfsOutputPin::open(RING_PIN).unwrap();

        let _dial = Dial::new::<SysfsInputPin>(nsa, nsi, input_send.clone());
        let _earpiece = Earpiece::new::<SysfsInputPin>(hook, input_send);
        state_machine.run();
    };
}
