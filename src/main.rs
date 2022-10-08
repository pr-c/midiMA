extern crate core;

mod config;
mod ma_interface;
mod midi_controller;
mod periodic_update_sender;

use crate::ma_interface::{FaderValue, LoginCredentials, Update};
use config::Config;
use ma_interface::MaInterface;
use midi_controller::MidiController;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver};
use tokio::sync::Mutex;
use tokio::time::Instant;

use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = Arc::new(get_config()?);
    let password_hash = md5::compute(config.console_password.clone());
    let login_credentials = LoginCredentials {
        username: config.console_username.clone(),
        password_hash: format!("{:x}", password_hash),
    };
    println!("Read config file");

    let (update_sender, update_receiver) = tokio::sync::mpsc::unbounded_channel();

    let midi_controller = MidiController::new(config.midi_devices.clone(), update_sender)?;


    main_loop(config, login_credentials, midi_controller, update_receiver).await
}

async fn main_loop(config: Arc<Config>, login_credentials: LoginCredentials, mut midi_controller: MidiController, update_receiver: UnboundedReceiver<Update>) -> Result<(), Box<dyn Error>> {
    let exec_value_receiver_mutex = Arc::new(Mutex::new(update_receiver));
    loop {
        let url = Url::parse(&("ws://".to_string() + &config.console_ip))?;
        let ma_mutex = Arc::new(Mutex::new(MaInterface::new(&url, &login_credentials).await?));
        println!("Connected to MA2 at {:?}", url.to_string());
        let forward_task = tokio::spawn(fader_to_ma_forward_loop(ma_mutex.clone(), exec_value_receiver_mutex.clone()));

        let last_message_received_instant = Arc::new(Mutex::new(Instant::now()));

        ma_poll_loop(config.ma_poll_interval, ma_mutex.clone(), &mut midi_controller, last_message_received_instant).await;
        forward_task.abort();
        println!("Network fail. Trying to reconnect...");
    }
}

async fn ma_poll_loop(poll_interval: u64, ma_mutex: Arc<Mutex<MaInterface>>, midi_controller: &mut MidiController, last_message_received_instant: Arc<Mutex<Instant>>) {
    let mut interval = tokio::time::interval(Duration::from_millis(poll_interval));
    loop {
        interval.tick().await;

        let mut ma_lock = ma_mutex.lock().await;
        let timeout_result = tokio::time::timeout(Duration::from_millis(2000), ma_lock.poll_fader_values()).await;
        drop(ma_lock);
        if let Ok(result) = timeout_result {
            if let Ok(values) = result {
                *last_message_received_instant.lock().await = Instant::now();
                for (i,value) in values.iter().enumerate() {
                    midi_controller.receive_update_from_ma(Update::FaderUpdate(FaderValue{
                        fader_value: *value,
                        exec_index: i as u8
                    })).await;
                }
            }
        } else {
            break;
        }
    }
}

async fn fader_to_ma_forward_loop(ma: Arc<Mutex<MaInterface>>, exec_value_receiver_mutex: Arc<Mutex<UnboundedReceiver<Update>>>) {
    let mut exec_value_receiver = exec_value_receiver_mutex.lock().await;
    while let Some(value) = exec_value_receiver.recv().await {
        ma.lock().await.send_update(value).unwrap();
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
