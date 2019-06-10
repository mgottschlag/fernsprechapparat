//! Types for GPIO input/output using the Linux sysfs interface.

pub struct SysfsInputPin {}

impl super::InputPin for SysfsInputPin {
    type Output = SysfsOutputPin;
    type Group = SysfsInputPinGroup;

    fn read(&self) -> bool {
        // TODO
        panic!("Not yet implemented.");
    }
    fn wait(&self) {
        // TODO
        panic!("Not yet implemented.");
    }

    fn create_set(_pins: Vec<Box<Self>>) -> Self::Group {
        // TODO
        panic!("Not yet implemented.");
    }
    fn into_output(self) -> Self::Output {
        // TODO
        panic!("Not yet implemented.");
    }
}

pub struct SysfsInputPinGroup {}

impl super::InputPinGroup for SysfsInputPinGroup {
    type Pin = SysfsInputPin;

    fn read(&self) -> u64 {
        // TODO
        panic!("Not yet implemented.");
    }
    fn wait(&self) {
        // TODO
        panic!("Not yet implemented.");
    }

    fn split(self) -> Vec<Box<Self::Pin>> {
        // TODO
        panic!("Not yet implemented.");
    }
}

pub struct SysfsOutputPin {}

impl super::OutputPin for SysfsOutputPin {
    type Input = SysfsInputPin;

    fn write(&self, _value: bool) {
        // TODO
        panic!("Not yet implemented.");
    }
    fn into_input(self) -> Self::Input {
        // TODO
        panic!("Not yet implemented.");
    }
}
