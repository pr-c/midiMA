pub mod ma_controlled_hardware;

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

pub struct MidiMessage {
    data: [u8; 3],
}

pub struct MidiController {
    motor_faders: Arc<Mutex<Vec<MotorFader>>>,
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

        let motor_faders_mutex = Arc::new(Mutex::new(Vec::new()));
        let mut lock = motor_faders_mutex.lock().await;
        for motor_fader_config in &config.motor_faders {
            lock.push(MotorFader::new(midi_tx_channel_sender.clone(), ma_sender.clone(), motor_fader_config));
        }
        drop(lock);

        let (tx, rx) = unbounded_channel();
        let midi_message_rx_task = tokio::spawn(Self::midi_rx_loop(rx, motor_faders_mutex.clone()));

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
            motor_faders: motor_faders_mutex,
            midi_message_rx_task,
            midi_message_tx_task,
        })
    }

    async fn midi_rx_loop(mut message_source: UnboundedReceiver<MidiMessage>, motor_faders_mutex: Arc<Mutex<Vec<MotorFader>>>) {
        while let Some(message) = &message_source.recv().await {
            let mut fader_lock = motor_faders_mutex.lock().await;
            for motor_fader in fader_lock.iter_mut() {
                motor_fader.set_value_from_midi(message).unwrap();
            }
        }
    }

    async fn midi_tx_loop(mut receiver: UnboundedReceiver<MidiMessage>, mut connection_tx: MidiOutputConnection) {
        while let Some(message) = receiver.recv().await {
            connection_tx.send(&message.data).unwrap();
        }
    }

    pub fn get_motor_faders_mutex(&self) -> Arc<Mutex<Vec<MotorFader>>> {
        self.motor_faders.clone()
    }

    fn find_midi_port<T: MidiIO>(midi: &mut T, port_name: &str) -> Result<T::Port, Box<dyn Error>> {
        for port in midi.ports() {
            if midi.port_name(&port)?.eq_ignore_ascii_case(port_name) {
                return Ok(port);
            }
        }
        Err("The midi input couldn't be found.")?
    }
}

impl Drop for MidiController {
    fn drop(&mut self) {
        self.midi_message_rx_task.abort();
        self.midi_message_tx_task.abort();
    }
}
