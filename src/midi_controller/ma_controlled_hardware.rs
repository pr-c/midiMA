pub mod motor_fader;
pub mod rotary_encoder;

use std::error::Error;
use crate::midi_controller::MidiMessage;

pub trait Hardware {
    fn set_value_from_midi(&mut self, message: &MidiMessage) -> Result<(), Box<dyn Error>>;
}