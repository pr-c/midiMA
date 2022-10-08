use std::error::Error;
use std::sync::Arc;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use crate::config::MidiDeviceConfig;
use crate::ma_interface::Update;
use model::DeviceModel;

use crate::midi_controller::midi_device::connection::Connection;
use crate::midi_controller::midi_device::feedback_handle::ModelFeedbackHandle;
use crate::midi_controller::midi_device::model::components::{MidiMessageReceiver, ReceivingError};
use crate::midi_controller::midi_message::MidiMessage;

mod model;
mod feedback_handle;
mod connection;

pub struct MidiDevice {
    _connection: Connection,
    midi_input_process_task: JoinHandle<()>,
    model_mutex: Arc<Mutex<DeviceModel>>,
}


impl MidiDevice {
    pub fn new(config: &MidiDeviceConfig, ma_feedback_handle: UnboundedSender<Update>) -> Result<Self, Box<dyn Error>> {
        let (connection, channels) = Connection::new(config)?;

        let feedback_handle = ModelFeedbackHandle::new(ma_feedback_handle, channels.sender);
        let model = DeviceModel::new(config.model.clone(), feedback_handle)?;

        let model_mutex = Arc::new(Mutex::new(model));
        let midi_input_process_task = tokio::spawn(Self::process_all_midi_inputs(channels.receiver, model_mutex.clone()));

        Ok(Self {
            _connection: connection,
            midi_input_process_task,
            model_mutex,
        })
    }

    pub async fn receive_update_from_ma(&mut self, update: Update) {
        let mut model = self.model_mutex.lock().await;
        let receive_result =model.receive_update_from_ma(update).await;
        drop(model);
        if let Err(e) = receive_result {
            self.handle_receive_error(e);
        }

    }

    async fn process_all_midi_inputs(mut source: UnboundedReceiver<MidiMessage>, model_mutex: Arc<Mutex<DeviceModel>>) {
        while let Some(message) = source.recv().await {
            let mut model = model_mutex.lock().await;
            let _result = model.receive_midi_message(message).await;
        }
    }

    fn handle_receive_error(&mut self, error: ReceivingError) {
        match error {
            ReceivingError::MidiError => {
                self.handle_midi_error();
            },
            ReceivingError::MaError => {

            }
        }
    }

    fn handle_midi_error(&mut self) {
        todo!();
    }
}


impl Drop for MidiDevice {
    fn drop(&mut self) {
        self.midi_input_process_task.abort();
    }
}


