use crate::config::ButtonConfig;
use crate::midi_controller::midi_message::MidiMessage;
use crate::midi_controller::midi_pattern::MidiPattern;

pub struct ButtonPattern {
    config: ButtonConfig,
}

impl ButtonPattern {
    pub fn new(config: ButtonConfig) -> Self {
        Self {
            config
        }
    }
}

impl MidiPattern for ButtonPattern {
    type State = bool;
    fn resolve_value_from_input(&self, message: &MidiMessage) -> Result<Self::State, ()> {
        if self.config.input_midi_byte_0 == message.data[0] && self.config.input_midi_byte_1 == message.data[1] {
            if message.data[2] == self.config.low_value.unwrap_or(0) {
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Err(())
        }
    }
    fn create_output_message_from_state(&self, value: &Self::State) -> MidiMessage {
        let val = {
            if *value {
                self.config.high_value.unwrap_or(127)
            } else {
                self.config.low_value.unwrap_or(0)
            }
        };
        MidiMessage {
            data: [self.config.output_midi_byte_0, self.config.output_midi_byte_1, val]
        }
    }
}