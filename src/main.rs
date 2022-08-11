mod config;
mod ma_connection;
use config::Config;
use ma_connection::MaInterface;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Reading config file");
    let config = get_config()?;

    let mut ma_interface: MaInterface = MaInterface::new(
        config.console_ip,
        config.console_username,
        config.console_password,
    )?;
    ma_interface.connect().await?;

    Ok(())
}

fn get_config() -> Result<Config, Box<dyn Error>> {
    let config_file_name = String::from("midiMA.json");
    let result = Config::read_from_config(&config_file_name);

    let config = match result {
        Ok(c) => c,
        Err(err) => {
            println!(
                "Error while reading config file {}: {}",
                config_file_name, err
            );
            Config::write_default_config_file(&config_file_name).unwrap_or(());
            Config::default()
        }
    };
    Ok(config)
}
