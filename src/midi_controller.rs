use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::config::{MidiControllerConfig, MotorFaderConfig};
use crate::MaInterface;
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

pub struct MotorFader {
    config: MotorFaderConfig,
    value: u8,
    tx: Arc<Mutex<MidiOutputConnection>>,
    ma_mutex: Arc<Mutex<MaInterface>>,
}

impl MotorFader {
    pub fn new(ma_mutex: Arc<Mutex<MaInterface>>, tx: Arc<Mutex<MidiOutputConnection>>, config: MotorFaderConfig) -> MotorFader {
        MotorFader { tx, value: 0, config, ma_mutex }
    }

    pub fn set_ma_value(&mut self, value: f32) -> Result<(), Box<dyn Error>> {
        self.set_value(self.ma_value_to_fader_value(value))
    }
    fn set_value(&mut self, value: u8) -> Result<(), Box<dyn Error>> {
        if value != self.value {
            self.value = value;
            self.send_value();
        }
        Ok(())
    }

    pub fn get_executor_index(&self) -> u8 {
        self.config.ma_executor_index
    }

    pub fn receive_midi_input(&mut self, message: &[u8]) {
        if message.len() < 3 {
            return;
        }
        if message[0] == self.config.input_midi_byte_0 && message[1] == self.config.input_midi_byte_1 {
            self.value = message[2];
            let mut ma_lock = self.ma_mutex.lock().unwrap();
            let ma_value = self.fader_value_to_ma_value(self.value);
            let _ = ma_lock.send_fader_value(self.config.ma_executor_index as u32, 0, ma_value);
            drop(ma_lock);
            if self.config.input_feedback.unwrap_or(true) {
                self.send_value();
            }
        }
    }

    fn fader_value_to_ma_value(&self, v: u8) -> f32 {
        ((v - self.config.min_value.unwrap_or(0)) as f32) / (self.config.max_value.unwrap_or(127) as f32)
    }

    fn ma_value_to_fader_value(&self, v: f32) -> u8 {
        (v * (self.config.max_value.unwrap_or(127) as f32)) as u8 + self.config.min_value.unwrap_or(0)
    }

    fn send_value(&self) {
        let mut lock = self.tx.lock().unwrap();
        let _ = lock.send(&[self.config.output_midi_byte_0, self.config.output_midi_byte_1, self.value]);
        drop(lock);
    }
}

pub struct MidiController {
    motor_faders: Arc<Mutex<Vec<MotorFader>>>,
    _connection_rx: MidiInputConnection<()>,
}

impl MidiController {
    pub fn new(config: MidiControllerConfig, ma_mutex: Arc<Mutex<MaInterface>>) -> Result<MidiController, Box<dyn Error>> {
        let mut midi_out = MidiOutput::new(&("MidiMA out ".to_owned() + &config.midi_out_port_name))?;
        let mut midi_in = MidiInput::new(&("MidiMA in ".to_owned() + &config.midi_in_port_name))?;

        let port_in = MidiController::find_midi_port(&mut midi_in, &config.midi_in_port_name)?;
        let port_out = MidiController::find_midi_port(&mut midi_out, &config.midi_out_port_name)?;

        let connection_tx = Arc::new(Mutex::new(midi_out.connect(&port_out, &config.midi_out_port_name)?));

        let motor_faders_mutex = Arc::new(Mutex::new(Vec::new()));
        let mut lock = motor_faders_mutex.lock().unwrap();
        for motor_fader_config in config.motor_faders {
            lock.push(MotorFader::new(ma_mutex.clone(), connection_tx.clone(), motor_fader_config));
        }
        drop(lock);

        let rx_motor_fader_mutex = motor_faders_mutex.clone();
        let connection_rx = midi_in.connect(
            &port_in,
            &config.midi_in_port_name,
            move |_stamp, message, _| {
                let mut lock = rx_motor_fader_mutex.lock().unwrap();
                for motor_fader in lock.iter_mut() {
                    motor_fader.receive_midi_input(message);
                }
                drop(lock);
            },
            (),
        )?;

        Ok(MidiController {
            _connection_rx: connection_rx,
            motor_faders: motor_faders_mutex,
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
