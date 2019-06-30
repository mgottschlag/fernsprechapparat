//! Application which implements SIP support for modded old telephones with
//! rotary dials.
//!
//! This program is intended for a mod of a FeTAp (Fernsprechtischapparat) of
//! the Deutsche Bundespost, but will likely work with any similar phones.

extern crate confy;
extern crate pjproject;
extern crate serde;
#[macro_use]
extern crate serde_derive;

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

use serde::{Deserialize, Serialize};

use std::sync::mpsc::channel;

#[derive(Debug, PartialEq)]
pub enum Event {
    Dialed(u32),
    EarpiecePickedUp,
    EarpiecePutDown,
    Registered,
    Unregistered,
}

#[derive(Serialize, Deserialize)]
struct Config {
    domain: String,
    user: String,
    password: String,
    cli: bool,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            domain: "".into(),
            user: "".into(),
            password: "".into(),
            cli: false,
        }
    }
}

// TODO: Correct GPIO numbers.
const NSA_PIN: usize = 1;
const NSI_PIN: usize = 2;
const RING_PIN: usize = 3;
const HOOK_PIN: usize = 4;

fn main() {
    let cfg: Config = confy::load("fernsprechapparat").unwrap();
    if cfg.password == "" {
        // Create config if it does not exist yet.
        confy::store("fernsprechapparat", cfg).ok();
        panic!("No valid configuration!");
    }

    let (input_send, input_recv) = channel();

    let sip = Sip::new(&cfg.domain, &cfg.user, &cfg.password);

    let mut state_machine = StateMachine::new(input_recv);

    if cfg.cli {
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
