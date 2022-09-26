pub mod motor_fader;

use std::error::Error;
use crate::ma_interface;
use crate::midi_controller::MidiMessage;

pub trait Hardware {
    fn set_value_from_midi(&mut self, message: &MidiMessage) -> Result<(), Box<dyn Error>>;
}

pub trait MaControlledHardware: Hardware {
    fn set_value_from_ma(&mut self, value: ma_interface::ValueChange) -> Result<(), Box<dyn Error>>;
}
