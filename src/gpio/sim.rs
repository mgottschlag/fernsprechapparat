//! Types for GPIO input/output simulated in software.
//!
//! These types can be used to simulate GPIO for unit tests.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub struct SimInputPin {
    env: Arc<SimEnvironmentState>,
    index: usize,
}

impl super::InputPin for SimInputPin {
    type Output = SimOutputPin;
    type Group = SimInputPinGroup;

    fn read(&self) -> bool {
        get_atomic_bit(&self.env.input_state, self.index)
    }
    fn wait(&self) {
        self.env.wait(1u64 << self.index);
    }
    fn wait_timeout(&self, timeout: Duration) -> bool {
        match self.env.wait_timeout(timeout, 1u64 << self.index) {
            Some(_) => true,
            None => false,
        }
    }

    fn create_group(pins: Vec<Box<Self>>) -> Self::Group {
        assert!(pins.len() > 0);
        let env = pins[0].env.clone();
        let mut indices = Vec::new();
        let mut mask = 0;
        for pin in pins {
            indices.push(pin.index);
            mask |= 1 << pin.index;
        }
        SimInputPinGroup { env, indices, mask }
    }
    fn into_output(self) -> Self::Output {
        // TODO
        panic!("Not yet implemented.");
    }
}

pub struct SimInputPinGroup {
    env: Arc<SimEnvironmentState>,
    indices: Vec<usize>,
    mask: u64,
}

impl SimInputPinGroup {
    fn indices_to_pin_bitmap(&self, indices: u64) -> u64 {
        let mut pins = 0;
        for (i, index) in self.indices.iter().enumerate() {
            if (indices & (1u64 << index)) != 0 {
                pins |= 1 << i;
            }
        }
        pins
    }
}

impl super::InputPinGroup for SimInputPinGroup {
    type Pin = SimInputPin;

    fn read(&self) -> u64 {
        let value = self.env.input_state.load(Ordering::SeqCst);
        let mut result = 0;
        for i in 0..self.indices.len() {
            if (value & (1 << self.indices[i])) != 0 {
                result |= 1 << i;
            }
        }
        result
    }
    fn wait(&self) {
        self.env.wait(self.mask);
    }
    fn wait_timeout(&self, timeout: Duration) -> Option<u64> {
        self.env
            .wait_timeout(timeout, self.mask)
            .map(|changed| self.indices_to_pin_bitmap(changed))
    }
    fn len(&self) -> usize {
        self.indices.len()
    }

    fn split(self) -> Vec<Box<Self::Pin>> {
        let env = self.env.clone();
        let mut pins = Vec::new();
        for index in self.indices {
            pins.push(Box::new(SimInputPin {
                env: env.clone(),
                index,
            }));
        }
        pins
    }
}

pub struct SimOutputPin {
    env: Arc<SimEnvironmentState>,
    index: usize,
}

impl super::OutputPin for SimOutputPin {
    type Input = SimInputPin;

    fn write(&self, value: bool) {
        set_atomic_bit(&self.env.output_state, self.index, value);
    }
    fn into_input(self) -> Self::Input {
        // TODO
        panic!("Not yet implemented.");
    }
}

struct SimEnvironmentState {
    input_state: AtomicU64,
    input_changed: Mutex<u64>,
    input_condvar: Condvar,
    output_state: AtomicU64,
    is_output: AtomicU64,
    can_change_type: AtomicU64,
}

impl SimEnvironmentState {
    fn wait(&self, mask: u64) {
        let mut input_changed = self.input_changed.lock().unwrap();
        loop {
            if (*input_changed & mask) != 0 {
                *input_changed &= !mask;
                return;
            }
            input_changed = self.input_condvar.wait(input_changed).unwrap();
        }
    }
    fn wait_timeout(&self, timeout: Duration, mask: u64) -> Option<u64> {
        let mut input_changed = self.input_changed.lock().unwrap();
        if (*input_changed & mask) != 0 {
            let changed = *input_changed & mask;
            *input_changed &= !mask;
            return Some(changed);
        }
        let (mut input_changed, timeout_res) = self
            .input_condvar
            .wait_timeout(input_changed, timeout)
            .unwrap();
        if timeout_res.timed_out() {
            return None;
        }

        if (*input_changed & mask) != 0 {
            let changed = *input_changed & mask;
            *input_changed &= !mask;
            return Some(changed);
        }

        // TODO: We do not repeat the call. Therefore, this function causes
        // spurious wakeups. Repeating the call requires more complex timeout
        // calculation.
        return None;
    }
}

/// Simulated device environment which can be used to generate input and output
/// pins.
pub struct SimEnvironment {
    state: Arc<SimEnvironmentState>,
}

impl SimEnvironment {
    pub fn new() -> Self {
        SimEnvironment {
            state: Arc::new(SimEnvironmentState {
                input_state: AtomicU64::new(0),
                input_changed: Mutex::new(0),
                input_condvar: Condvar::new(),
                output_state: AtomicU64::new(0),
                is_output: AtomicU64::new(0),
                can_change_type: AtomicU64::new(0),
            }),
        }
    }

    pub fn create_input_pin(&self, index: usize, can_change_type: bool) -> SimInputPin {
        set_atomic_bit(&self.state.can_change_type, index, can_change_type);
        set_atomic_bit(&self.state.is_output, index, false);
        set_atomic_bit(&self.state.input_state, index, false);
        set_atomic_bit(&self.state.output_state, index, false);
        *self.state.input_changed.lock().unwrap() &= !(1 << index);
        SimInputPin {
            env: self.state.clone(),
            index,
        }
    }
    pub fn create_output_pin(&self, index: usize, can_change_type: bool) -> SimOutputPin {
        set_atomic_bit(&self.state.can_change_type, index, can_change_type);
        set_atomic_bit(&self.state.is_output, index, true);
        set_atomic_bit(&self.state.input_state, index, false);
        set_atomic_bit(&self.state.output_state, index, false);
        *self.state.input_changed.lock().unwrap() &= !(1 << index);
        SimOutputPin {
            env: self.state.clone(),
            index,
        }
    }

    pub fn write_input(&self, index: usize, value: bool) {
        set_atomic_bit(&self.state.input_state, index, value);
        let mut input_changed = self.state.input_changed.lock().unwrap();
        *input_changed |= 1 << index;
        self.state.input_condvar.notify_all();
    }
    pub fn read_output(&self, index: usize) -> bool {
        get_atomic_bit(&self.state.output_state, index)
    }
}

fn set_atomic_bit(a: &AtomicU64, bit: usize, value: bool) {
    let mask = 1u64 << bit;
    if value {
        a.fetch_or(mask, Ordering::SeqCst);
    } else {
        a.fetch_and(!mask, Ordering::SeqCst);
    }
}

fn get_atomic_bit(a: &AtomicU64, bit: usize) -> bool {
    (a.load(Ordering::SeqCst) & (1 << bit)) != 0
}
