use std::error::Error;
use async_trait::async_trait;
use crate::config::ButtonConfig;
use crate::ma_interface::{ButtonValue, Update};
use crate::midi_controller::MaUpdateReceiver;
use crate::midi_controller::midi_device::model::{ModelFeedbackHandle, MidiDeviceComponent, MidiMessageReceiver};
use crate::midi_controller::midi_message::MidiMessage;
use crate::midi_controller::midi_pattern::button_pattern::ButtonPattern;
use crate::midi_controller::midi_pattern::MidiPattern;
use crate::Update::ButtonUpdate;

pub struct Button {
    pattern: ButtonPattern,
    current_state: bool,
    feedback_handle: ModelFeedbackHandle,
    config: ButtonConfig,
}

impl Button {
    fn process_midi_input(&mut self, state: bool) {
        if self.current_state != state {
            self.current_state = state;
            let _ = self.feedback_handle.midi.send(self.pattern.create_output_message_from_state(&self.current_state));
            let _ = self.feedback_handle.ma.send(self.get_update());
        }
    }
    fn process_ma_input(&mut self, state: bool) {
        if self.current_state != state {
            self.current_state = state;
            let _ = self.feedback_handle.midi.send(self.pattern.create_output_message_from_state(&self.current_state));
        }
    }
    fn get_update(&self) -> Update {
        Update::ButtonUpdate(ButtonValue {
            exec_index: self.config.ma_executor_index,
            button_value: self.current_state,
            position: self.config.position,
        })
    }
}


impl MidiDeviceComponent for Button {
    type Config = ButtonConfig;
    fn new(config: Self::Config, feedback_handle: ModelFeedbackHandle) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            pattern: ButtonPattern::new(config.clone()),
            current_state: false,
            feedback_handle,
            config,
        })
    }
}


#[async_trait]
impl MidiMessageReceiver for Button {
    async fn receive_midi_message(&mut self, message: MidiMessage) -> Result<(), ()> {
        if let Ok(value) = self.pattern.resolve_value_from_input(&message) {
            self.process_midi_input(value);
            Ok(())
        } else {
            Err(())
        }
    }
}

#[async_trait]
impl MaUpdateReceiver for Button {
    async fn receive_update_from_ma(&mut self, update: Update) {
        if let ButtonUpdate(button_value) = update {
            if button_value.exec_index == self.config.ma_executor_index && button_value.position == self.config.position {
                self.process_ma_input(button_value.button_value);
            }
        }
    }
}

