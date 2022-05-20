use std::thread;
use std::time::Duration;

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::serial::Uart;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::serial;

use pms5003::PmsAQIData;

fn process_pms<UART: Uart, TX: OutputPin, RX: InputPin>(serial: &mut serial::Serial<UART, TX, RX>) {
    println!("Read PMS5003 message");

    match PmsAQIData::read(serial).as_ref() {
        Ok(data) => {
            println!("Received PMS data: {:?}", data);
        }
        Err(e) => {
            println!("Error reading PMS data: {:?}", e);
        }
    }
}

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    println!("Starting up");

    thread::sleep(Duration::from_millis(500));

    let mut serial = {
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

        println!("PMS5003 serial setup");
        serial
    };

    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        thread::sleep(Duration::from_millis(500));

        process_pms(&mut serial);
    }
}
