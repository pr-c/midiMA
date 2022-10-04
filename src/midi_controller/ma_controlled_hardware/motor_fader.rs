use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use crate::config::MotorFaderConfig;
use crate::ma_interface::FaderValue;
use crate::midi_controller::ma_controlled_hardware::Hardware;
use crate::midi_controller::ma_controlled_hardware::periodic_update_sender::PeriodicUpdateSender;
use crate::midi_controller::MidiMessage;


use async_trait::async_trait;

pub struct MotorFader {
    config: MotorFaderConfig,
    value: u8,
    midi_tx: UnboundedSender<MidiMessage>,
    periodic_sender: PeriodicUpdateSender<FaderValue>,
}

impl MotorFader {
    pub fn new(midi_tx: UnboundedSender<MidiMessage>, ma_tx: UnboundedSender<FaderValue>, config: &MotorFaderConfig) -> Result<MotorFader, Box<dyn Error>> {
        let periodic_sender = PeriodicUpdateSender::new(ma_tx, Duration::from_millis(50))?;

        Ok(MotorFader {
            midi_tx,
            value: 0,
            config: config.clone(),
            periodic_sender,
        })
    }

    pub fn get_executor_index(&self) -> u8 {
        self.config.ma_executor_index
    }

    pub fn set_value_from_ma(&mut self, fader_value: f32) -> Result<(), Box<dyn Error>> {
        let new_value = Self::ma_value_to_fader_value(&self.config, fader_value);
        if new_value != self.value {
            self.value = new_value;
            self.send_value_to_midi()?;
        }
        Ok(())
    }

    fn send_value_to_midi(&self) -> Result<(), Box<dyn Error>> {
        let send_result = self.midi_tx.send(MidiMessage {
            data: [self.config.output_midi_byte_0, self.config.output_midi_byte_1, self.value]
        });
        if send_result.is_ok() {
            Ok(())
        } else {
            Err("Midi device closed".into())
        }
    }

    async fn send_value_to_ma(&mut self) -> Result<(), Box<dyn Error>> {
        let ma_value = self.fader_value_to_ma_value(self.value);
        self.periodic_sender.set_value(FaderValue {
            exec_index: self.config.ma_executor_index,
            page_index: 0,
            fader_value: ma_value,
        }).await
    }

    fn fader_value_to_ma_value(&self, v: u8) -> f32 {
        ((v - self.config.min_value.unwrap_or(0)) as f32) / (self.config.max_value.unwrap_or(127) as f32)
    }

    fn ma_value_to_fader_value(config: &MotorFaderConfig, v: f32) -> u8 {
        (v * (config.max_value.unwrap_or(127) as f32)).round() as u8 + config.min_value.unwrap_or(0)
    }
}

#[async_trait]
impl Hardware for MotorFader {
    async fn set_value_from_midi(&mut self, message: &MidiMessage) -> Result<(), Box<dyn Error>> {
        if message.data[0] == self.config.input_midi_byte_0 && message.data[1] == self.config.input_midi_byte_1 {
            let new_value = message.data[2];
            if self.value != new_value {
                self.value = new_value;
                self.send_value_to_ma().await?;
                if self.config.input_feedback.unwrap_or(true) {
                    self.send_value_to_midi()?;
                }
            }
        }
        Ok(())
    }
}