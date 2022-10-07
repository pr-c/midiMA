pub mod midi_pattern;
pub mod midi_device;
pub mod midi_message;

use async_trait::async_trait;
use std::error::Error;
use crate::config::MidiDeviceConfig;
use crate::midi_controller::midi_device::MidiDevice;
use tokio::sync::mpsc::UnboundedSender;
use crate::ma_interface::Update;


pub struct MidiController {
    midi_devices: Vec<MidiDevice>,
}

impl MidiController {
    pub fn new(configs: Vec<MidiDeviceConfig>, ma_feedback_handle: UnboundedSender<Update>) -> Result<MidiController, Box<dyn Error>> {
        let mut midi_devices = Vec::new();
        for config in &configs {
            midi_devices.push(
                MidiDevice::new(config, ma_feedback_handle.clone())?
            );
        }

        Ok(MidiController {
            midi_devices
        })
    }
}


#[async_trait]
pub trait MaUpdateReceiver {
    async fn receive_update_from_ma(&mut self, update: Update);
}

#[async_trait]
impl MaUpdateReceiver for MidiController {
    async fn receive_update_from_ma(&mut self, update: Update) {
        for device in self.midi_devices.iter_mut() {
            device.receive_update_from_ma(update).await;
        }
    }
}