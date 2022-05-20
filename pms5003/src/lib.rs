#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

use embedded_hal::nb;
use embedded_hal::serial::nb::Read;

use heapless::Deque;
#[cfg(feature = "std")]
use thiserror::Error;

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "std", derive(Error))]
pub enum ParseError {
    #[cfg_attr(feature = "std", error("Invalid checksum on PMS5003 data frame"))]
    InvalidChecksum,
    #[cfg_attr(feature = "std", error("Invalid start byte (should be 0x42)"))]
    InvalidStartByte,
}

// #[derive(Debug)]
// #[cfg_attr(feature = "std", derive(Error))]
// pub enum Error<E> {
//     #[cfg_attr(feature = "std", error(transparent))]
//     ParseError(#[cfg_attr(feature = "std", from)] ParseError),
//     #[cfg_attr(feature = "std", error("Error reading PMS5003 data: {0}"))]
//     ReadError(#[cfg_attr(feature = "std", from)] E)
// }

#[derive(Debug)]
pub enum Error<E> {
    ParseError(ParseError),
    ReadError(E),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PmsAQIData {
    /// How long this data chunk is
    pub framelen: u16,
    /// Standard PM1.0
    pub pm10_standard: u16,
    /// Standard PM2.5
    pub pm25_standard: u16,
    /// Standard PM10.0
    pub pm100_standard: u16,
    /// Environmental PM1.0
    pub pm10_env: u16,
    /// Environmental PM2.5
    pub pm25_env: u16,
    /// Environmental PM10.0
    pub pm100_env: u16,
    /// 0.3um Particle Count
    pub particles_03um: u16,
    /// 0.5um Particle Count
    pub particles_05um: u16,
    /// 1.0um Particle Count
    pub particles_10um: u16,
    /// 2.5um Particle Count
    pub particles_25um: u16,
    /// 5.0um Particle Count
    pub particles_50um: u16,
    /// 10.0um Particle Count
    pub particles_100um: u16,
    /// Unused
    pub unused: u16,
    /// Packet checksum
    pub checksum: u16,
}

struct BufReader<R, const N: usize>(R, Deque<u8, N>);

impl<R, const N: usize> BufReader<R, N>
where
    R: Read<u8>,
{
    pub fn new(reader: R) -> Self {
        BufReader(reader, Deque::new())
    }

    pub fn fill(&mut self) -> Result<(), R::Error> {
        while !self.1.is_full() {
            self.1.push_back(nb::block!(self.0.read())?).unwrap();
        }

        Ok(())
    }

    pub fn read(&mut self) -> Result<u8, R::Error> {
        self.fill()?;
        let ret = self.1.pop_front().unwrap();
        self.fill()?;
        Ok(ret)
    }

    pub fn filled_buffer(&mut self) -> Result<[u8; N], R::Error> {
        self.fill()?;
        let mut it = self.1.iter();
        Ok([(); N].map(|_| *it.next().unwrap()))
    }
}

fn get_u16_from_be(bytes: &[u8], i: usize) -> u16 {
    let idx = i * 2;
    u16::from_be_bytes([bytes[idx], bytes[idx + 1]])
}

impl PmsAQIData {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        if bytes[0] != 0x42 {
            return Err(ParseError::InvalidStartByte);
        }

        let mut actual_checksum: u16 = 0;

        // Only checksum everything up to the expected checksum (last 2 bytes)
        for b in &bytes[..30] {
            actual_checksum = actual_checksum.wrapping_add(*b as u16);
        }

        let checksum = get_u16_from_be(&bytes, 15);

        if checksum != actual_checksum {
            return Err(ParseError::InvalidChecksum);
        }

        Ok(PmsAQIData {
            framelen: get_u16_from_be(bytes, 1),
            pm10_standard: get_u16_from_be(bytes, 2),
            pm25_standard: get_u16_from_be(bytes, 3),
            pm100_standard: get_u16_from_be(bytes, 4),
            pm10_env: get_u16_from_be(bytes, 5),
            pm25_env: get_u16_from_be(bytes, 6),
            pm100_env: get_u16_from_be(bytes, 7),
            particles_03um: get_u16_from_be(bytes, 8),
            particles_05um: get_u16_from_be(bytes, 9),
            particles_10um: get_u16_from_be(bytes, 10),
            particles_25um: get_u16_from_be(bytes, 11),
            particles_50um: get_u16_from_be(bytes, 12),
            particles_100um: get_u16_from_be(bytes, 13),
            unused: get_u16_from_be(bytes, 14),
            checksum,
        })
    }

    pub fn read<R>(rx: R) -> Result<PmsAQIData, Error<R::Error>>
    where
        R: Read<u8>,
    {
        let mut reader: BufReader<R, 32> = BufReader::new(rx);

        loop {
            let raw_msg = reader.filled_buffer().map_err(Error::ReadError)?;

            match PmsAQIData::from_bytes(&raw_msg) {
                Ok(data) => {
                    return Ok(data);
                }
                Err(e) => {
                    #[cfg(feature = "std")]
                    println!("Failed to parse PMS5003 message: {:?}", e);
                    reader.read().map_err(Error::ReadError)?;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{PmsAQIData, ParseError};

    #[test]
    fn test_read_correct_frame() {
        let raw_msg = vec![
            66, 77, 0, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 150, 0, 48, 0, 16, 0, 2, 0, 0, 0,
            0, 151, 0, 2, 26,
        ];
        assert_eq!(raw_msg.len(), 32);

        let expected_msg = PmsAQIData {
            framelen: 28,
            pm10_standard: 0,
            pm25_standard: 0,
            pm100_standard: 0,
            pm10_env: 0,
            pm25_env: 0,
            pm100_env: 0,
            particles_03um: 150,
            particles_05um: 48,
            particles_10um: 16,
            particles_25um: 2,
            particles_50um: 0,
            particles_100um: 0,
            unused: 38656,
            checksum: 538,
        };

        let msg = PmsAQIData::from_bytes(&raw_msg).expect("Parses correct message");

        assert_eq!(msg, expected_msg);
    }

    #[test]
    fn test_read_bad_checksum_frame() {
        let raw_msg = vec![
            66, 77, 0, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 150, 0, 48, 0, 16, 0, 2, 0, 0, 0,
            0, 151, 0, 2, 13,
        ];

        let e = PmsAQIData::from_bytes(&raw_msg).expect_err("Bad checksum should fail");
        assert_eq!(e, ParseError::InvalidChecksum);
    }

    #[test]
    fn test_read_invalid_start_byte_frame() {
        let raw_msg = vec![
            0, 77, 0, 28, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 150, 0, 48, 0, 16, 0, 2, 0, 0, 0,
            0, 151, 0, 2, 26,
        ];

        let e = PmsAQIData::from_bytes(&raw_msg).expect_err("Bad checksum should fail");
        assert_eq!(e, ParseError::InvalidStartByte);
    }
}
