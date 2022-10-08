use std::error::Error;
use async_trait::async_trait;
use crate::config::{DeviceModelConfig};
use crate::ma_interface::Update;

use crate::midi_controller::midi_message::MidiMessage;
use crate::midi_controller::midi_device::model::components::button::Button;
use crate::midi_controller::midi_device::model::components::fader::Fader;
use crate::midi_controller::midi_device::ModelFeedbackHandle;
use crate::midi_controller::midi_device::model::components::{MaUpdateReceiver, MidiDeviceComponent, MidiMessageReceiver, ReceivingError, ReceivingState};

pub mod components;

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

    pub async fn receive_update_from_ma(&mut self, update: Update) -> Result<(), ReceivingError>{
        for fader in &mut self.faders {
            fader.receive_update_from_ma(update).await?;
        }
        for button in &mut self.buttons {
            button.receive_update_from_ma(update).await?;
        }
        Ok(())
    }
}


#[async_trait]
impl MidiMessageReceiver for DeviceModel {
    async fn receive_midi_message(&mut self, message: MidiMessage) -> Result<ReceivingState, ReceivingError> {
        for fader in &mut self.faders {
            if fader.receive_midi_message(message).await? == ReceivingState::Consumed {
                return Ok(ReceivingState::Consumed);
            }
        }
        for button in &mut self.buttons {
            if button.receive_midi_message(message).await? == ReceivingState::Consumed {
                return Ok(ReceivingState::Consumed);
            }
        }
        Ok(ReceivingState::Pass)
    }
}

