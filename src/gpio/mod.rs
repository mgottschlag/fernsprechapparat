//! Types for GPIO input/output.

use std::time::Duration;

#[allow(dead_code)]
pub mod sim;
pub mod sysfs;

/// Single GPIO input pin with support for interrupts.
pub trait InputPin {
    /// Type of the corresponding output pin (see `InputPin::into_output`).
    type Output: OutputPin;
    /// Group of multiple input pins read together (see `InputPin::create_set`).
    type Group: InputPinGroup;

    /// Reads the value at the pin.
    fn read(&self) -> bool;
    /// Waits for a pin value change interrupt.
    fn wait(&self);
    /// Waits for a pin value change interrupt or until a timeout expires.
    ///
    /// The function returns false if no interrupt arrived within the specified
    /// duration.
    fn wait_timeout(&self, timeout: Duration) -> bool;

    /// Groups multiple input pins into an input pin set so that a single call
    /// can be used to wait for changes of multiple pins.
    fn create_group(pins: Vec<Box<Self>>) -> Self::Group;
    /// Converts this input pin into an output pin.
    fn into_output(self) -> Self::Output;
}

/// Group of multiple input pins which supports reading multiple pin values and
/// waiting for multiple change interrupts.
pub trait InputPinGroup {
    /// Type of the individual input pins.
    type Pin: InputPin;

    /// Reads the input of all the pins in the set.
    ///
    /// Note that the call is not atomic and the individual pins are evaluated
    /// sequentially.
    fn read(&self) -> u64;
    /// Waits for a value change of one of the pins.
    fn wait(&self);
    /// Waits for a pin value change interrupt or until a timeout expires.
    ///
    /// The function either returns the number of the pin which triggered the
    /// interrupt or `None` if the timeout expired.
    fn wait_timeout(&self, timeout: Duration) -> Option<u64>;
    /// Returns the number of pins in this group.
    fn len(&self) -> usize;

    /// Splits the set into its individual pins.
    fn split(self) -> Vec<Box<Self::Pin>>;
}

/// Single GPIO output pin.
pub trait OutputPin {
    /// Type of the corresponding input pin (see `OutputPin::into_input`).
    type Input: InputPin;

    /// Sets the value of the pin.
    fn write(&self, value: bool);

    /// Converts this output pin into an input pin.
    fn into_input(self) -> Self::Input;
}
