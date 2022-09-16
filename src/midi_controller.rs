mod motor_fader;

use std::error::Error;
use std::sync::{Arc};

use motor_fader::MotorFader;
use tokio::sync::{mpsc, Mutex};
use crate::config::MidiControllerConfig;
use crate::MaInterface;
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;

pub struct MidiMessage {
    data: [u8; 3],
}

pub struct MidiController {
    motor_faders: Arc<Mutex<Vec<MotorFader>>>,
    receiver_task_handle: JoinHandle<()>,
    _connection_rx: MidiInputConnection<()>,
}

async fn receiver_task(mut message_source: UnboundedReceiver<MidiMessage>, motor_faders_mutex: Arc<Mutex<Vec<MotorFader>>>) {
    loop {
        let message_option = message_source.recv().await;
        if let Some(message) = &message_option {
            let mut fader_lock = motor_faders_mutex.lock().await;
            for motor_fader in fader_lock.iter_mut() {
                motor_fader.set_value_from_midi(message).await;
            }
        } else {
            break;
        }
    }
}

impl MidiController {
    pub async fn new(config: MidiControllerConfig, ma_mutex: Arc<Mutex<MaInterface>>) -> Result<MidiController, Box<dyn Error>> {
        let mut midi_out = MidiOutput::new(&("MidiMA out ".to_owned() + &config.midi_out_port_name))?;
        let mut midi_in = MidiInput::new(&("MidiMA in ".to_owned() + &config.midi_in_port_name))?;

        let port_in = MidiController::find_midi_port(&mut midi_in, &config.midi_in_port_name)?;
        let port_out = MidiController::find_midi_port(&mut midi_out, &config.midi_out_port_name)?;

        let connection_tx = Arc::new(Mutex::new(midi_out.connect(&port_out, &config.midi_out_port_name)?));

        let motor_faders_mutex = Arc::new(Mutex::new(Vec::new()));
        let mut lock = motor_faders_mutex.lock().await;
        for motor_fader_config in config.motor_faders {
            lock.push(MotorFader::new(connection_tx.clone(), ma_mutex.clone(), motor_fader_config));
        }
        drop(lock);

        let (tx, rx) = mpsc::unbounded_channel();
        let receiver_task_handle = tokio::spawn(receiver_task(rx, motor_faders_mutex.clone()));


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
            receiver_task_handle,
        })
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
        self.receiver_task_handle.abort();
    }
}
