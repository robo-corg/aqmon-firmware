use std::str;
use std::thread;
use std::time::Duration;

use byteorder::BigEndian;
use byteorder::ByteOrder;
use byteorder::ReadBytesExt;
use embedded_hal::nb;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_hal::serial::nb::{Read, Write};

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::serial;

#[derive(Debug, Default, Clone)]
struct PmsAQIData {
    /// How long this data chunk is
    framelen: u16,
    /// Standard PM1.0
    pm10_standard: u16,
    /// Standard PM2.5
    pm25_standard: u16,
    /// Standard PM10.0
    pm100_standard: u16,
    /// Environmental PM1.0
    pm10_env: u16,
    /// Environmental PM2.5
    pm25_env: u16,
    /// Environmental PM10.0
    pm100_env: u16,
    /// 0.3um Particle Count
    particles_03um: u16,
    /// 0.5um Particle Count
    particles_05um: u16,
    /// 1.0um Particle Count
    particles_10um: u16,
    /// 2.5um Particle Count
    particles_25um: u16,
    /// 5.0um Particle Count
    particles_50um: u16,
    /// 10.0um Particle Count
    particles_100um: u16,
    /// Unused
    unused: u16,
    /// Packet checksum
    checksum: u16,
}

impl PmsAQIData {
    fn read<R>(mut rx: R) -> PmsAQIData
    where
        R: Read<u8>,
    {
        loop {
            if nb::block!(rx.read()).unwrap() == 0x42 {
                break;
            }

            thread::sleep(Duration::from_millis(500));
        }

        let b1: u8 = 0x42;
        let b2 = nb::block!(rx.read()).unwrap();

        let raw_data = [(); 30].map(|_| nb::block!(rx.read()).unwrap());

        let mut sum: u16 = 0;
        sum = sum.wrapping_add(b1 as u16);
        sum = sum.wrapping_add(b2 as u16);

        for b in &raw_data[0..28] {
            sum = sum.wrapping_add(*b as u16);
        }

        let mut raw_data_cursor = raw_data.as_slice();

        let data = PmsAQIData {
            framelen: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            pm10_standard: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            pm25_standard: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            pm100_standard: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            pm10_env: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            pm25_env: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            pm100_env: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            particles_03um: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            particles_05um: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            particles_10um: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            particles_25um: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            particles_50um: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            particles_100um: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            unused: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
            checksum: raw_data_cursor.read_u16::<BigEndian>().unwrap(),
        };

        if data.checksum != sum {
            dbg!(data.checksum, sum);
        }

        data
    }
}

fn main() {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    println!("Starting up");

    thread::sleep(Duration::from_millis(500));

    let peripherals = Peripherals::take().unwrap();
    let tx = peripherals.pins.gpio5;
    let rx = peripherals.pins.gpio4;

    let config = serial::config::Config::default().baudrate(Hertz(9600));
    let mut serial: serial::Serial<serial::UART1, _, _> = serial::Serial::new(
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

    println!("Serial setup");

    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        thread::sleep(Duration::from_millis(500));

        let data = PmsAQIData::read(&mut serial);
        dbg!(data);
    }
}
