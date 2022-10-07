use std::error::Error;
use async_trait::async_trait;
use crate::config::ButtonConfig;
use crate::ma_interface::Update;
use crate::midi_controller::MaUpdateReceiver;
use crate::midi_controller::midi_device::model::{ModelFeedbackHandle, MidiDeviceComponent, MidiMessageReceiver};
use crate::midi_controller::midi_message::MidiMessage;
use crate::midi_controller::midi_pattern::button_pattern::ButtonPattern;
use crate::midi_controller::midi_pattern::MidiPattern;

pub struct Button {
    config: ButtonConfig,
    pattern: ButtonPattern,
    current_value: bool,
    feedback_handle: ModelFeedbackHandle,
}

impl Button {
    fn process_midi_input(&mut self, state: bool) {}
}


impl MidiDeviceComponent for Button {
    type Config = ButtonConfig;
    fn new(config: Self::Config, feedback_handle: ModelFeedbackHandle) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            pattern: ButtonPattern::new(config.clone()),
            config,
            current_value: false,
            feedback_handle,
        })
    }
}


#[async_trait]
impl MidiMessageReceiver for Button {
    async fn receive_midi_message(&mut self, message: &MidiMessage) -> Result<(), ()> {
        if let Ok(value) = self.pattern.resolve_value_from_input(message) {
            self.process_midi_input(value);
            Ok(())
        } else {
            Err(())
        }
    }
}

#[async_trait]
impl MaUpdateReceiver for Button {
    async fn receive_update_from_ma(&mut self, update: Update) {}
}

