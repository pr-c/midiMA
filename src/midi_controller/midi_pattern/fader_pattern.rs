use crate::config::MotorFaderConfig;
use crate::midi_controller::midi_message::MidiMessage;
use crate::midi_controller::midi_pattern::MidiPattern;

pub struct FaderPattern {
    config: MotorFaderConfig,
}

impl FaderPattern {
    pub fn new(config: MotorFaderConfig) -> Self {
        Self { config }
    }
}

impl MidiPattern for FaderPattern {
    type State = u8;

    fn resolve_value_from_input(&self, message: &MidiMessage) -> Result<Self::State, ()> {
        if message.data[0] == self.config.input_midi_byte_0 && message.data[1] == self.config.input_midi_byte_1 {
            Ok(message.data[2])
        } else {
            Err(())
        }
    }

    fn create_output_message_from_state(&self, value: Self::State) -> MidiMessage {
        MidiMessage {
            data: [self.config.output_midi_byte_0, self.config.output_midi_byte_1, value]
        }
    }
}