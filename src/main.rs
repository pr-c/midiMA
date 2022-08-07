mod config;
use config::Config;
use std::error::Error;

fn main() {
    println!("-------------------- midiMa --------------------");
    match run() {
        Ok(_) => {},
        Err(e) => {println!("Failed with error: {}", e)}
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    println!("Reading config file");

    let config_file_name = String::from("midiMA.json");
    let result = Config::read_from_config(&config_file_name);
    
    let config = match result {
        Ok(c) => c,
        Err(err) => {
            println!("Error while reading config file {}: {}", config_file_name, err);
            Config::write_default_config_file(&config_file_name).unwrap();
            Config::default()
        }
    };

    Ok(())
}

