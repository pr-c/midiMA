use std::{error::Error, fs, fs::File, io::Write};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    console_ip: String,
    console_port: u16
}

impl Config {
    pub fn default() -> Config {
        Config { console_ip: String::from("127.0.0.1"), console_port: 2794 }
    }

    pub fn write_default_config_file(filename: &str) -> Result<(), Box<dyn Error>> {
        let default_config = Config::default();
        let serialized = serde_json::to_string(&default_config)?;

        let mut file = File::create(filename)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    pub fn read_from_config(file_name: &str) -> Result<Config,Box<dyn Error>> {
        let content = fs::read_to_string(file_name)?;
        let config : Config = serde_json::from_str(&content)?;
        Ok(config)
    }
}
