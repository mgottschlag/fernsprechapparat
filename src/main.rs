//! Application which implements SIP support for modded old telephones with
//! rotary dials.
//!
//! This program is intended for a mod of a FeTAp (Fernsprechtischapparat) of
//! the Deutsche Bundespost, but will likely work with any similar phones.

mod dial;
mod gpio;

#[derive(Debug, PartialEq)]
pub enum Event {
    Dialed(u32),
}

fn main() {
    println!("Hello, world!");
}
