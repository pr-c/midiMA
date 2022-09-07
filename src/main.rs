extern crate core;

mod config;
mod ma_connection;
mod midi_controller;
use crate::ma_connection::LoginCredentials;
use config::Config;
use ma_connection::MaInterface;
use midi_controller::MidiController;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Reading config file");
    let config = get_config()?;
    let login_credentials = LoginCredentials {
        username: config.console_username,
        password: config.console_password,
    };

    let url = Url::parse(&("ws://".to_string() + &config.console_ip))?;

    let mut midi = MidiController::new()?;
    loop {
        println!("New Connection");
        let mut ma = MaInterface::new(&url, &login_credentials).await?;
        loop {
            sleep(Duration::from_millis(10));
            if let Ok(fader_values) = ma.get_fader_values(10, 0).await {
                for (i, value) in fader_values.iter().enumerate() {
                    if i > 8 {
                        break;
                    }
                    midi.set_fader_position(i as u8, *value)?;
                }
            } else {
                break;
            }
        }
    }
}

fn get_config() -> Result<Config, Box<dyn Error>> {
    let config_file_name = String::from("midiMA.json");
    let result = Config::read_from_config(&config_file_name);

    let config = match result {
        Ok(c) => c,
        Err(err) => {
            println!("Error while reading config file {}: {}", config_file_name, err);
            Config::write_default_config_file(&config_file_name).unwrap_or(());
            Config::default()
        }
    };
    Ok(config)
}
