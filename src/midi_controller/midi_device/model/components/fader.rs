use std::error::Error;
use std::time::Duration;
use async_trait::async_trait;
use crate::config::MotorFaderConfig;
use crate::FaderValue;
use crate::ma_interface::Update;
use crate::ma_interface::Update::FaderUpdate;
use crate::midi_controller::midi_message::MidiMessage;
use crate::midi_controller::midi_pattern::fader_pattern::FaderPattern;
use crate::midi_controller::midi_pattern::MidiPattern;
use crate::midi_controller::midi_device::model::{ModelFeedbackHandle, MidiMessageReceiver};
use crate::midi_controller::midi_device::model::components::{MaUpdateReceiver, MidiDeviceComponent, ReceivingError, ReceivingState};
use crate::periodic_update_sender::PeriodicUpdateSender;


pub struct Fader {
    config: MotorFaderConfig,
    pattern: FaderPattern,
    current_state: u8,
    ma_update_sender: PeriodicUpdateSender<Update>,
    midi_update_sender: PeriodicUpdateSender<MidiMessage>,
}

impl Fader {
    fn fader_value_to_ma_value(&self, v: u8) -> f32 {
        ((v - self.config.min_value.unwrap_or(0)) as f32) / (self.config.max_value.unwrap_or(127) as f32)
    }
    fn ma_value_to_fader_value(&self, v: f32) -> u8 {
        (v * (self.config.max_value.unwrap_or(127) as f32)).round() as u8 + self.config.min_value.unwrap_or(0)
    }

    async fn process_midi_input(&mut self, state: u8)  -> Result<(), ReceivingError>{
        if self.current_state != state {
            self.current_state = state;
            self.send_state_to_ma().await?;
            self.send_state_to_midi().await?;
        }
        Ok(())
    }

    async fn process_ma_input(&mut self, value: u8)  -> Result<(), ReceivingError>{
        if self.current_state != value {
            self.current_state = value;
            self.send_state_to_midi().await?;
        }
        Ok(())
    }

    async fn send_state_to_midi(&mut self) -> Result<(), ReceivingError>{
        let midi_send_result = self.midi_update_sender.set_value(self.pattern.create_output_message_from_state(&self.current_state)).await;
        if midi_send_result.is_err() {
            return Err(ReceivingError::MidiError);
        }
        Ok(())
    }

    async fn send_state_to_ma(&mut self) -> Result<(), ReceivingError> {
        let ma_send_result = self.ma_update_sender.set_value(self.get_update()).await;
        if ma_send_result.is_err() {
            return Err(ReceivingError::MaError);
        }
        Ok(())
    }

    fn get_update(&self) -> Update {
        Update::FaderUpdate(FaderValue {
            exec_index: self.config.ma_executor_index,
            fader_value: self.fader_value_to_ma_value(self.current_state),
        })
    }
}

impl MidiDeviceComponent for Fader {
    type Config = MotorFaderConfig;
    fn new(config: Self::Config, feedback_handle: ModelFeedbackHandle) -> Result<Self, Box<dyn Error>> {
        let ma_update_sender = PeriodicUpdateSender::new(feedback_handle.ma, Duration::from_millis(50))?;
        let midi_update_sender = PeriodicUpdateSender::new(feedback_handle.midi, Duration::from_millis(50))?;
        Ok(Self {
            pattern: FaderPattern::new(config.clone()),
            current_state: 0,
            config,
            ma_update_sender,
            midi_update_sender,
        })
    }
}

#[async_trait]
impl MaUpdateReceiver for Fader {
    async fn receive_update_from_ma(&mut self, update: Update) -> Result<(), ReceivingError> {
        if let FaderUpdate(value) = update {
            if value.exec_index == self.config.ma_executor_index && !self.ma_update_sender.is_sending() {
                let midi_value = self.ma_value_to_fader_value(value.fader_value);
                self.process_ma_input(midi_value).await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl MidiMessageReceiver for Fader {
    async fn receive_midi_message(&mut self, message: MidiMessage) -> Result<ReceivingState, ReceivingError>{
        if let Ok(value) = self.pattern.resolve_value_from_input(&message) {
            self.process_midi_input(value).await?;
            Ok(ReceivingState::Consumed)
        } else {
            Ok(ReceivingState::Pass)
        }
    }
}