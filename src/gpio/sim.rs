//! Types for GPIO input/output simulated in software.
//!
//! These types can be used to simulate GPIO for unit tests.

pub struct SimInputPin {}

impl super::InputPin for SimInputPin {
    type Output = SimOutputPin;
    type Group = SimInputPinGroup;

    fn read(&self) -> bool {
        // TODO
        panic!("Not yet implemented.");
    }
    fn wait(&self) {
        // TODO
        panic!("Not yet implemented.");
    }

    fn create_group(_pins: Vec<Box<Self>>) -> Self::Group {
        // TODO
        panic!("Not yet implemented.");
    }
    fn into_output(self) -> Self::Output {
        // TODO
        panic!("Not yet implemented.");
    }
}

pub struct SimInputPinGroup {}

impl super::InputPinGroup for SimInputPinGroup {
    type Pin = SimInputPin;

    fn read(&self) -> u64 {
        // TODO
        panic!("Not yet implemented.");
    }
    fn wait(&self) {
        // TODO
        panic!("Not yet implemented.");
    }
    fn len(&self) -> usize {
        // TODO
        panic!("Not yet implemented.");
    }

    fn split(self) -> Vec<Box<Self::Pin>> {
        // TODO
        panic!("Not yet implemented.");
    }
}

pub struct SimOutputPin {}

impl super::OutputPin for SimOutputPin {
    type Input = SimInputPin;

    fn write(&self, _value: bool) {
        // TODO
        panic!("Not yet implemented.");
    }
    fn into_input(self) -> Self::Input {
        // TODO
        panic!("Not yet implemented.");
    }
}

struct SimEnvironmentState {
    // TODO
}

/// Simulated device environment which can be used to generate input and output
/// pins.
pub struct SimEnvironment {
    // TODO
}

impl SimEnvironment {
    pub fn new() -> Self {
        SimEnvironment {
            // TODO
        }
    }

    pub fn create_input_pin(&self, index: usize, can_change_type: bool) -> SimInputPin {
        // TODO
        panic!("Not yet implemented.");
    }
    pub fn create_output_pin(&self, index: usize, can_change_type: bool) -> SimOutputPin {
        // TODO
        panic!("Not yet implemented.");
    }

    pub fn write_input(&self, index: usize, value: bool) {
        // TODO
        panic!("Not yet implemented.");
    }
    pub fn read_output(&self, index: usize) -> bool {
        // TODO
        panic!("Not yet implemented.");
    }
}
