pub mod fader;
pub mod button;

use std::error::Error;
use async_trait::async_trait;
use crate::midi_controller::midi_device::feedback_handle::ModelFeedbackHandle;
use crate::midi_controller::midi_message::MidiMessage;
use crate::Update;

pub trait MidiDeviceComponent<T: MidiMessageReceiver = Self> {
    type Config;
    fn new(config: Self::Config, feedback_handle: ModelFeedbackHandle) -> Result<Self, Box<dyn Error>>
        where Self: std::marker::Sized;
}

#[derive(PartialEq)]
pub enum ReceivingState {
    Consumed,
    Pass,
}

#[derive(PartialEq)]
pub enum ReceivingError {
    MaError,
    MidiError,
}


#[async_trait]
pub trait MidiMessageReceiver {
    async fn receive_midi_message(&mut self, message: MidiMessage) -> Result<ReceivingState, ReceivingError>;
}

#[async_trait]
pub trait MaUpdateReceiver {
    async fn receive_update_from_ma(&mut self, update: Update) -> Result<(), ReceivingError>;
}

