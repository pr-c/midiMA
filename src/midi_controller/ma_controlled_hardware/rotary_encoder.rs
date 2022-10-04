use std::error::Error;
use crate::midi_controller::ma_controlled_hardware::Hardware;
use crate::midi_controller::MidiMessage;
use async_trait::async_trait;
use crate::config::RotaryEncoderConfig;

pub struct RotaryEncoder {
    config: RotaryEncoderConfig
}


impl RotaryEncoder {
    pub fn new(config: &RotaryEncoderConfig) -> Self {
        Self{
            config: config.clone()
        }
    }
}

#[async_trait]
impl Hardware for RotaryEncoder {
    async fn set_value_from_midi(&mut self, message: &MidiMessage) -> Result<(), Box<dyn Error>> {
        println!("{:?}", message.data);
        Ok(())
    }
}