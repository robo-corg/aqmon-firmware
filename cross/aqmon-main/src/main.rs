use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::serial::Uart;
use esp_idf_svc::nvs::EspDefaultNvs;
use esp_idf_svc::nvs_storage::EspNvsStorage;
use esp_idf_svc::sysloop::EspSysLoopStack;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::serial;
use embedded_svc::httpd::registry::Registry;
use esp_idf_svc::httpd;
use esp_idf_svc::netif::EspNetifStack;
use anyhow::bail;
use esp_idf_svc::wifi::EspWifi;
use once_cell::sync::OnceCell;
use tracing::info;

use pms5003::PmsAQIData;
use tracing::trace;

use crate::command::process_commands;
use crate::config::WifiConfig;

pub static ESP_NVS_STORAGE: OnceCell<Mutex<EspNvsStorage>> = OnceCell::new();

mod config;
mod command;



fn process_pms<UART: Uart, TX: OutputPin, RX: InputPin>(mut serial: serial::Serial<UART, TX, RX>) {
    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        thread::sleep(Duration::from_millis(500));

        match PmsAQIData::read(&mut serial).as_ref() {
            Ok(data) => {
                trace!("Received PMS data: {:?}", data);
            }
            Err(e) => {
                info!("Error reading PMS data: {:?}", e);
            }
        }
    }
}

fn wifi(
    wifi_config: WifiConfig,
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>, anyhow::Error> {
    use esp_idf_svc::wifi::*;
    use embedded_svc::wifi::{Configuration, ClientStatus, ClientIpStatus, ClientConnectionStatus, ApStatus, ApIpStatus, Status, ClientConfiguration, AccessPointConfiguration, Wifi};


    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == wifi_config.ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            &wifi_config.ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            &wifi_config.ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: wifi_config.ssid,
            password:wifi_config.password,
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected: {}", ip_settings.ip);

        //ping(&ip_settings)?;
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

//#[allow(unused_variables)]
fn start_httpd() -> Result<httpd::Server, anyhow::Error> {
    let server = httpd::ServerRegistry::new()
        .at("/")
        .get(|_| Ok("Hello from Rust!".into()))?;


    //#[cfg(esp32s2)]
    //let server = httpd_ulp_endpoints(server, mutex)?;

    server.start(&Default::default())
}

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    tracing_subscriber::fmt::init();

    info!("Starting up");

    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new().unwrap());
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new().unwrap());
    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new().unwrap());

    ESP_NVS_STORAGE.set(Mutex::new(
        EspNvsStorage::new_default(default_nvs.clone(), "aqmon-app", true).unwrap()
    )).map_err(|_| anyhow::anyhow!("Storage already installed")).unwrap();

    thread::sleep(Duration::from_millis(500));

    //let _http_server = start_httpd().unwrap();

    let serial = {
        let peripherals = Peripherals::take().unwrap();
        let tx = peripherals.pins.gpio5;
        let rx = peripherals.pins.gpio4;

        let config = serial::config::Config::default().baudrate(Hertz(9600));
        let serial: serial::Serial<serial::UART1, _, _> = serial::Serial::new(
            peripherals.uart1,
            serial::Pins {
                tx,
                rx,
                cts: None,
                rts: None,
            },
            config,
        )
        .unwrap();

        info!("PMS5003 serial setup");
        serial
    };


    thread::spawn(move || process_pms(serial));

    process_commands();
}
