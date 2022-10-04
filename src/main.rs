extern crate core;

mod config;
mod ma_interface;
mod midi_controller;

use crate::ma_interface::{FaderValue, LoginCredentials};
use config::Config;
use ma_interface::MaInterface;
use midi_controller::MidiController;
use std::error::Error;
use std::sync::{Arc};
use std::time::Duration;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
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

    let (executor_value_sender, executor_value_receiver) = tokio::sync::mpsc::unbounded_channel();

    let midi_controllers = init_midi_controllers(&config, executor_value_sender).await?;
    println!("Connected to {} midi device[s].", midi_controllers.len());

    main_loop(config, login_credentials, midi_controllers, executor_value_receiver).await
}

async fn main_loop(config: Arc<Config>, login_credentials: LoginCredentials, midi_controllers: Arc<Vec<MidiController>>, executor_value_receiver: UnboundedReceiver<FaderValue>) -> Result<(), Box<dyn Error>> {
    let exec_value_receiver_mutex = Arc::new(Mutex::new(executor_value_receiver));
    loop {
        let url = Url::parse(&("ws://".to_string() + &config.console_ip))?;
        let ma_mutex = Arc::new(Mutex::new(MaInterface::new(&url, &login_credentials).await?));
        println!("Connected to MA2 at {:?}", url.to_string());
        let forward_task = tokio::spawn(fader_to_ma_forward_loop(ma_mutex.clone(), exec_value_receiver_mutex.clone()));

        let last_message_received_instant = Arc::new(Mutex::new(Instant::now()));

        ma_poll_loop(config.ma_poll_interval, ma_mutex.clone(), &midi_controllers, last_message_received_instant).await;
        forward_task.abort();
        println!("Network fail. Trying to reconnect...");
    }
}

async fn init_midi_controllers(config: &Arc<Config>, executor_value_sender: UnboundedSender<FaderValue>) -> Result<Arc<Vec<MidiController>>, Box<dyn Error>> {
    let mut midi_controllers = Vec::new();
    for midi_controller_config in &config.midi_devices {
        midi_controllers.push(MidiController::new((*midi_controller_config).clone(), executor_value_sender.clone()).await?);
    }
    if midi_controllers.is_empty() {
        return Err("No midi devices configured".into());
    }
    Ok(Arc::new(midi_controllers))
}

async fn ma_poll_loop(poll_interval: u64, ma_mutex: Arc<Mutex<MaInterface>>, midi_controllers: &Vec<MidiController>, last_message_received_instant: Arc<Mutex<Instant>>) {
    let mut interval = tokio::time::interval(Duration::from_millis(poll_interval));
    loop {
        interval.tick().await;

        let mut ma_lock = ma_mutex.lock().await;
        let timeout_result = tokio::time::timeout(Duration::from_millis(2000), ma_lock.poll_fader_values()).await;
        drop(ma_lock);
        if let Ok(result) = timeout_result {
            if let Ok(values) = result {
                *last_message_received_instant.lock().await = Instant::now();
                for controller in midi_controllers {
                    let controls_mutex = controller.get_controls();
                    let mut controls_lock = controls_mutex.lock().await;
                    for fader in controls_lock.motor_faders.iter_mut() {
                        fader.set_value_from_ma(values[fader.get_executor_index() as usize]).unwrap();
                    }
                }
            }
        } else {
            break;
        }
    }
}

async fn fader_to_ma_forward_loop(ma: Arc<Mutex<MaInterface>>, exec_value_receiver_mutex: Arc<Mutex<UnboundedReceiver<FaderValue>>>) {
    let mut exec_value_receiver = exec_value_receiver_mutex.lock().await;
    while let Some(value) = exec_value_receiver.recv().await {
        ma.lock().await.send_executor_value(&value).unwrap();
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
