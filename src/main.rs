extern crate core;

mod config;
mod ma_connection;
mod midi_controller;
use crate::ma_connection::LoginCredentials;
use config::Config;
use ma_connection::MaInterface;
use midi_controller::MidiController;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Reading config file");
    let config = get_config()?;
    let hash = md5::compute(config.console_password);
    let login_credentials = LoginCredentials {
        username: config.console_username,
        password_hash: format!("{:x}", hash),
    };

    let url = Url::parse(&("ws://".to_string() + &config.console_ip))?;
    let ma_mutex = Arc::new(Mutex::new(MaInterface::new(&url, &login_credentials).await?));
    let mut midi_controllers = Vec::new();
    for midi_controller_config in config.midi_devices {
        midi_controllers.push(MidiController::new(midi_controller_config, ma_mutex.clone())?);
    }
    if midi_controllers.is_empty() {
        return Err("No midi devices configured".into());
    }
    println!("Connected to MA2 Server and {} midi device[s].", midi_controllers.len());
    let mut interval = tokio::time::interval(Duration::from_millis(config.ma_poll_interval.unwrap_or(10)));
    loop {
        interval.tick().await;
        let mut ma_lock = ma_mutex.lock().unwrap();
        let result = ma_lock.poll_fader_values().await;
        drop(ma_lock);
        if let Ok(values) = result {
            for controller in &midi_controllers {
                let fader_mutex = controller.get_motor_faders_mutex();
                let mut fader_lock = fader_mutex.lock().unwrap();
                for fader in fader_lock.iter_mut() {
                    let _ = fader.set_ma_value(values[fader.get_executor_index() as usize]);
                }
                drop(fader_lock);
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
