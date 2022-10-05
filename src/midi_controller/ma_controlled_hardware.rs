pub mod motor_fader;
pub mod rotary_encoder;

use async_trait::async_trait;

use std::error::Error;
use crate::midi_controller::MidiMessage;

#[async_trait]
pub trait Hardware {
    async fn set_value_from_midi(&mut self, message: &MidiMessage) -> Result<(), Box<dyn Error>>;
}