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
        Err(())
    }
    fn create_output_message_from_state(&self, value: Self::State) -> MidiMessage {
        todo!()
    }
}