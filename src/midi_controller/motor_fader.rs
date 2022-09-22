use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use midir::MidiOutputConnection;
use tokio::task::JoinHandle;
use crate::config::MotorFaderConfig;
use crate::MaInterface;
use tokio::sync::Mutex;
use crate::ma_connection::ExecutorValue;
use crate::midi_controller::MidiMessage;

pub struct MotorFader {
    config: Arc<MotorFaderConfig>,
    value: u8,
    midi_tx: Arc<Mutex<MidiOutputConnection>>,
    ma_tx: Arc<Mutex<MaInterface>>,
    ma_sender_task: Option<JoinHandle<()>>,
    ma_sending_value: Arc<Mutex<Option<u8>>>,
}

impl MotorFader {
    pub fn new(midi_tx: Arc<Mutex<MidiOutputConnection>>, ma_tx: Arc<Mutex<MaInterface>>, config: MotorFaderConfig) -> MotorFader {
        MotorFader {
            midi_tx,
            ma_tx,
            value: 0,
            config: Arc::new(config),
            ma_sender_task: None,
            ma_sending_value: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn set_value_from_ma(&mut self, value: f32) -> Result<(), Box<dyn Error>> {
        let new_value = MotorFader::ma_value_to_fader_value(&self.config, value);
        if new_value != self.value {
            self.value = new_value;
            self.send_value_to_midi().await?;
        }
        Ok(())
    }


    pub async fn set_value_from_midi(&mut self, message: &MidiMessage) {
        if message.data[0] == self.config.input_midi_byte_0 && message.data[1] == self.config.input_midi_byte_1 {
            let new_value = message.data[2];
            if self.value != new_value {
                self.value = new_value;

                let mut val_lock = self.ma_sending_value.lock().await;
                *val_lock = Some(self.value);
                drop(val_lock);

                if self.ma_sender_task.is_none() {
                    self.start_ma_update();
                } else if let Some(handle) = &self.ma_sender_task {
                    if handle.is_finished() {
                        self.start_ma_update();
                    }
                }
                if self.config.input_feedback.unwrap_or(true) {
                    let _ = self.send_value_to_midi().await;
                }
            }
        }
    }

    fn start_ma_update(&mut self) {
        self.ma_sender_task = Some(tokio::spawn(MotorFader::ma_update_loop(self.ma_sending_value.clone(), self.config.clone(), self.ma_tx.clone())));
    }

    pub fn get_executor_index(&self) -> u8 {
        self.config.ma_executor_index
    }

    pub async fn ma_update_loop(new_value: Arc<Mutex<Option<u8>>>, config: Arc<MotorFaderConfig>, ma: Arc<Mutex<MaInterface>>) {
        let mut interval = tokio::time::interval(Duration::from_millis(50));
        loop {
            interval.tick().await;
            let mut val_lock = new_value.lock().await;
            if let Some(value) = *val_lock {
                let value_clone = value;
                *val_lock = None;
                let mut ma_lock = ma.lock().await;
                MotorFader::send_value_to_ma(&config, &mut ma_lock, value_clone);
            } else {
                break;
            }
        }
    }

    async fn send_value_to_midi(&self) -> Result<(), Box<dyn Error>> {
        let mut lock = self.midi_tx.lock().await;
        lock.send(&[self.config.output_midi_byte_0, self.config.output_midi_byte_1, self.value])?;
        Ok(())
    }

    fn send_value_to_ma(config: &Arc<MotorFaderConfig>, ma: &mut MaInterface, value: u8) {
        let ma_value = MotorFader::fader_value_to_ma_value(config, value);
        let _ = ma.send_executor_value(&ExecutorValue::new(config.ma_executor_index as u32, 0, ma_value));
    }

    fn fader_value_to_ma_value(config: &Arc<MotorFaderConfig>, v: u8) -> f32 {
        ((v - config.min_value.unwrap_or(0)) as f32) / (config.max_value.unwrap_or(127) as f32)
    }

    fn ma_value_to_fader_value(config: &Arc<MotorFaderConfig>, v: f32) -> u8 {
        (v * (config.max_value.unwrap_or(127) as f32)).round() as u8 + config.min_value.unwrap_or(0)
    }
}

impl Drop for MotorFader {
    fn drop(&mut self) {
        if let Some(join_handle) = &mut self.ma_sender_task {
            join_handle.abort();
        }
    }
}