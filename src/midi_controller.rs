pub mod ma_controlled_hardware;
mod periodic_update_sender;

use std::error::Error;
use std::sync::Arc;

use ma_controlled_hardware::motor_fader::MotorFader;
use tokio::sync::Mutex;
use crate::config::MidiControllerConfig;
use midir::{MidiInput, MidiInputConnection, MidiIO, MidiOutput, MidiOutputConnection};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use crate::ma_interface::FaderValue;
use crate::midi_controller::ma_controlled_hardware::Hardware;
use crate::midi_controller::ma_controlled_hardware::rotary_encoder::RotaryEncoder;

pub struct MidiMessage {
    data: [u8; 3],
}

pub struct Controls {
    pub motor_faders: Vec<MotorFader>,
    pub rotary_encoders: Vec<RotaryEncoder>
}

impl Controls {
    pub fn new_empty() -> Self {
        Self {
            motor_faders: Vec::new(),
            rotary_encoders: Vec::new()
        }
    }
}

pub struct MidiController {
    controls: Arc<Mutex<Controls>>,
    midi_message_rx_task: JoinHandle<()>,
    midi_message_tx_task: JoinHandle<()>,
    _connection_rx: MidiInputConnection<()>,
}

impl MidiController {
    pub async fn new(config: MidiControllerConfig, ma_sender: UnboundedSender<FaderValue>) -> Result<MidiController, Box<dyn Error>> {
        let mut midi_out = MidiOutput::new(&("MidiMA out ".to_owned() + &config.midi_out_port_name))?;
        let mut midi_in = MidiInput::new(&("MidiMA in ".to_owned() + &config.midi_in_port_name))?;

        let port_in = MidiController::find_midi_port(&mut midi_in, &config.midi_in_port_name)?;
        let port_out = MidiController::find_midi_port(&mut midi_out, &config.midi_out_port_name)?;

        let connection_tx = midi_out.connect(&port_out, &config.midi_out_port_name)?;

        let (midi_tx_channel_sender, midi_tx_channel_receiver) = unbounded_channel();
        let midi_message_tx_task = tokio::spawn(Self::midi_tx_loop(midi_tx_channel_receiver, connection_tx));

        let mut controls = Controls::new_empty();
        for motor_fader_config in &config.motor_faders {
            controls.motor_faders.push(MotorFader::new(midi_tx_channel_sender.clone(), ma_sender.clone(), motor_fader_config)?);
        }
        for rotary_encoder_config in &config.rotary_encoders {
            controls.rotary_encoders.push(RotaryEncoder::new(rotary_encoder_config));
        }

        let controls_mutex = Arc::new(Mutex::new(controls));

        let (tx, rx) = unbounded_channel();
        let midi_message_rx_task = tokio::spawn(Self::midi_rx_loop(rx, controls_mutex.clone()));

        let connection_rx = midi_in.connect(
            &port_in,
            &config.midi_in_port_name,
            move |_stamp, message, _| {
                let midi_data = message.try_into();
                if let Ok(data) = midi_data {
                    let _ = tx.send(MidiMessage {
                        data
                    });
                }
            },
            (),
        )?;

        Ok(MidiController {
            _connection_rx: connection_rx,
            controls: controls_mutex,
            midi_message_rx_task,
            midi_message_tx_task,
        })
    }

    async fn midi_rx_loop(mut message_source: UnboundedReceiver<MidiMessage>, controls: Arc<Mutex<Controls>>) {
        while let Some(message) = &message_source.recv().await {
            let mut controls_lock = controls.lock().await;
            for motor_fader in controls_lock.motor_faders.iter_mut() {
                motor_fader.set_value_from_midi(message).await.unwrap();
            }
            for rotary_encoder in controls_lock.rotary_encoders.iter_mut() {
                rotary_encoder.set_value_from_midi(message).await.unwrap();
            }
        }
    }

    async fn midi_tx_loop(mut receiver: UnboundedReceiver<MidiMessage>, mut connection_tx: MidiOutputConnection) {
        while let Some(message) = receiver.recv().await {
            connection_tx.send(&message.data).unwrap();
        }
    }

    pub fn get_controls(&self) -> Arc<Mutex<Controls>> {
        self.controls.clone()
    }

    fn find_midi_port<T: MidiIO>(midi: &mut T, port_name: &str) -> Result<T::Port, Box<dyn Error>> {
        for port in midi.ports() {
            if midi.port_name(&port)?.eq_ignore_ascii_case(port_name) {
                return Ok(port);
            }
        }
        Err("The midi port couldn't be found.")?
    }
}

impl Drop for MidiController {
    fn drop(&mut self) {
        self.midi_message_rx_task.abort();
        self.midi_message_tx_task.abort();
    }
}
