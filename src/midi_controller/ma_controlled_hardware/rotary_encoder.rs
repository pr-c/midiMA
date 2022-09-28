use std::error::Error;
use crate::midi_controller::ma_controlled_hardware::Hardware;
use crate::midi_controller::MidiMessage;

pub struct RotaryEncoder {

}


impl RotaryEncoder {
    pub fn new() -> Self {
        Self{

        }
    }
}

impl Hardware for RotaryEncoder {
    fn set_value_from_midi(&mut self, message: &MidiMessage) -> Result<(), Box<dyn Error>> {
        println!("{:?}", message.data);
        Ok(())
    }
}