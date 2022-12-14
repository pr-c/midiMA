use serde::{Deserialize, Serialize};
use std::{error::Error, fs, fs::File, io::Write};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub console_ip: String,
    pub console_username: String,
    pub console_password: String,
    pub midi_devices: Vec<MidiDeviceConfig>,
    pub ma_poll_interval: u64,
}

impl Config {
    pub fn default() -> Config {
        Config {
            console_ip: String::from("192.168.178.71"),
            console_username: String::from("remote"),
            console_password: String::from("remote"),
            midi_devices: Vec::new(),
            ma_poll_interval: 10,
        }
    }

    pub fn write_default_config_file(filename: &str) -> Result<(), Box<dyn Error>> {
        let default_config = Config::default();
        let serialized = serde_json::to_string(&default_config)?;

        let mut file = File::create(filename)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn read_from_config(file_name: &str) -> Result<Config, Box<dyn Error>> {
        let content = fs::read_to_string(file_name)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MidiDeviceConfig {
    pub midi_in_port_name: String,
    pub midi_out_port_name: String,
    pub model: DeviceModelConfig
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeviceModelConfig {
    pub motor_faders: Vec<MotorFaderConfig>,
    pub rotary_encoders: Vec<RotaryEncoderConfig>,
    pub buttons: Vec<ButtonConfig>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RotaryEncoderConfig {

}


#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ButtonPosition {
    Top,
    Mid,
    Bottom
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ButtonConfig {
    pub input_midi_byte_0: u8,
    pub input_midi_byte_1: u8,
    pub output_midi_byte_0: u8,
    pub output_midi_byte_1: u8,
    pub low_value: Option<u8>,
    pub high_value: Option<u8>,
    pub input_feedback: Option<bool>,
    pub ma_executor_index: u8,
    pub position: ButtonPosition
}



#[derive(Serialize, Deserialize, Clone)]
pub struct MotorFaderConfig {
    pub input_midi_byte_0: u8,
    pub input_midi_byte_1: u8,
    pub output_midi_byte_0: u8,
    pub output_midi_byte_1: u8,
    pub min_value: Option<u8>,
    pub max_value: Option<u8>,
    pub input_feedback: Option<bool>,
    pub ma_executor_index: u8,
}
