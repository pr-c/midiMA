pub mod fader_pattern;
pub mod button_pattern;

use crate::midi_controller::midi_message::MidiMessage;

pub trait MidiPattern {
    type State;
    fn resolve_value_from_input(&self, message: &MidiMessage) -> Result<Self::State, ()>;
    fn create_output_message_from_state(&self, state: &Self::State) -> MidiMessage;
}