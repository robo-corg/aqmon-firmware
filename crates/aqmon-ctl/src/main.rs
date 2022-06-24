use std::{time::Duration, io::{BufReader, BufRead}};

use clap::{Parser, Subcommand};
use messages::config::WifiConfig;

#[derive(Debug, Parser)]
#[clap(name = "aqmon-ctl")]
#[clap(about = "Control attached aqmon", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[clap(subcommand)]
    Configure(ConfigCommands),
}

#[derive(Debug, Subcommand)]
enum ConfigCommands {
    Wifi {
        #[clap(long)]
        ssid: String,
        #[clap(long)]
        password: String,
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let ports = serialport::available_ports().expect("No ports found!");
    for p in ports.iter() {
        println!("{}", p.port_name);
    }

    let serial_device_name = ports[0].port_name.as_str();

    //dbg!(serial_device);

    let mut port = serialport::new(serial_device_name, 115_200)
        .timeout(Duration::from_millis(100))
        .open().expect("Failed to open port");

    match cli.command {
        Commands::Configure(config_command) => match config_command {
            ConfigCommands::Wifi { ssid, password } => {
                let wifi_config = WifiConfig {
                    ssid,
                    password
                };

                let mut payload = serde_json::to_string(&messages::command::Command::SetWifiConfig(wifi_config))?;
                payload.push('\n');
                //let payload_bytes = payload.into_bytes();

                println!("Sending config: {:?}", payload);


                port.write_all(payload.as_bytes())?;
                port.flush()?;

                println!("Payload sent!");

                let mut port_buf = BufReader::new(port);

                let mut line_buf = String::new();

                //loop {
                    port_buf.read_line(&mut line_buf)?;
                    println!("{}", &line_buf);
                //}
            },
        },
    }

    Ok(())
}
