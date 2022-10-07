use std::error::Error;
use async_trait::async_trait;
use crate::config::{DeviceModelConfig};
use crate::ma_interface::Update;
use crate::midi_controller::MaUpdateReceiver;

use crate::midi_controller::midi_message::MidiMessage;
use crate::midi_controller::midi_device::model::button::Button;
use crate::midi_controller::midi_device::model::fader::Fader;
use crate::midi_controller::midi_device::ModelFeedbackHandle;

pub mod fader;
pub mod button;

pub struct DeviceModel {
    faders: Vec<Fader>,
    buttons: Vec<Button>,
}

impl DeviceModel {
    pub fn new(config: DeviceModelConfig, feedback_handle: ModelFeedbackHandle) -> Result<Self, Box<dyn Error>> {
        let mut faders = Vec::with_capacity(config.motor_faders.len());
        for fader_config in config.motor_faders {
            let fader = Fader::new(fader_config, feedback_handle.clone())?;
            faders.push(fader);
        }
        let mut buttons = Vec::with_capacity(config.buttons.len());
        for button_config in config.buttons {
            let button = Button::new(button_config, feedback_handle.clone())?;
            buttons.push(button);
        }
        Ok(DeviceModel {
            faders,
            buttons,
        })
    }
}

pub trait MidiDeviceComponent<T: MidiMessageReceiver = Self> {
    type Config;
    fn new(config: Self::Config, feedback_handle: ModelFeedbackHandle) -> Result<Self, Box<dyn Error>>
        where Self: std::marker::Sized;
}

#[async_trait]
pub trait MidiMessageReceiver {
    async fn receive_midi_message(&mut self, message: MidiMessage) -> Result<(), ()>;
}


#[async_trait]
impl MidiMessageReceiver for DeviceModel {
    async fn receive_midi_message(&mut self, message: MidiMessage) -> Result<(), ()> {
        for fader in &mut self.faders {
            if fader.receive_midi_message(message).await.is_ok() {
                return Ok(());
            }
        }
        for button in &mut self.buttons {
            if button.receive_midi_message(message).await.is_ok() {
                return Ok(());
            }
        }
        Err(())
    }
}

#[async_trait]
impl MaUpdateReceiver for DeviceModel {
    async fn receive_update_from_ma(&mut self, update: Update) {
        for fader in &mut self.faders {
            fader.receive_update_from_ma(update).await;
        }
        for button in &mut self.buttons {
            button.receive_update_from_ma(update).await;
        }
    }
}

