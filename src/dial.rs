use super::gpio::InputPinGroup;
use super::Event;

use std::sync::mpsc::Sender;

pub struct Dial<Pins: InputPinGroup> {
    pins: Pins,
    sender: Sender<Event>,
}

impl<Pins: InputPinGroup> Dial<Pins> {
    fn new(pins: Pins, sender: Sender<Event>) -> Self {
        // TODO
        Self { pins, sender }
    }
}

impl<Pins: InputPinGroup> Drop for Dial<Pins> {
    fn drop(&mut self) {
        // TODO: Stop the thread.
    }
}
