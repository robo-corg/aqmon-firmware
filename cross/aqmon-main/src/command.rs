use std::{io::{self, Read}, thread, time::Duration, ops::DerefMut};
use std::str;

use embedded_svc::storage::Storage;

use serde::{Deserialize};
use tracing::{info, error};

use crate::config::{Config, WifiConfig};
use crate::ESP_NVS_STORAGE;


//pub use messages::command::Command;

#[derive(Debug, Deserialize)]
pub enum Command {
    SetWifiConfig(WifiConfig)
}


fn run_command<S: Storage>(command: Command, storage: &mut S) -> Result<(), anyhow::Error> {
    match command {
        Command::SetWifiConfig(wifi_config) => {
            wifi_config.save(storage)?;
            info!("Wifi config updated to use `{}`", &wifi_config.ssid);
        },
    }


    Ok(())
}


fn run_command_str<S: Storage>(storage: &mut S, command_str: &str) -> Result<(), anyhow::Error> {
    let command: Command = serde_json::from_str(command_str)?;

    run_command(command, storage)?;

    Ok(())
}

pub fn process_commands() {
    let mut line_buf = Vec::new();

    loop {
        //let mut buffer = String::new();

        {
            let mut buffer = [0; 16];
            let mut stdin = io::stdin().lock();
            match stdin.read(&mut buffer) {
                Ok(bytes_read) => {
                    for b in &buffer[..bytes_read] {
                        line_buf.push(*b);

                        if *b == b'\n' {
                            let mut should_clear = false;

                            if let Some(line_buf_str) = str::from_utf8(line_buf.as_slice()).ok() {
                                if !line_buf_str.ends_with('\n') {
                                    continue;
                                }

                                should_clear = true;

                                trace!("Got command str: {:?}", line_buf_str);

                                let mut storage_locked = ESP_NVS_STORAGE.wait().lock().unwrap();

                                match run_command_str(storage_locked.deref_mut(), &line_buf_str[..line_buf_str.len()-1]) {
                                    Ok(_) => {

                                    },
                                    Err(e) => {
                                        error!("Error executing: {:?}", e);
                                    },
                                }
                            }

                            if should_clear {
                                line_buf.clear();
                            }
                        }
                    }
                },
                Err(_) => {},
            }
        }

        thread::sleep(Duration::from_millis(500));
    }
}