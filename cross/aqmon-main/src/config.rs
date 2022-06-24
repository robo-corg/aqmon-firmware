use anyhow::Error;
use embedded_svc::storage::Storage;
use serde::{Serialize, Deserialize};

//pub use messages::config::WifiConfig;

const WIFI_CONFIG_KEY: &str = "WIFI_CONFIG";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WifiConfig {
    pub ssid: String,
    pub password: String
}


pub trait Config: Sized {
    fn load<S: Storage>(storage: &S) -> Result<Self, Error>;
    fn save<S: Storage>(&self, storage: &mut S) -> Result<(), Error>;
}


impl Config for WifiConfig {
    fn load<S: Storage>(storage: &S) -> Result<Self, Error> {
        let maybe_raw_config_data = storage.get_raw(WIFI_CONFIG_KEY)?;

        if let Some(raw_config_data) = maybe_raw_config_data {
            let wifi_config: WifiConfig = postcard::from_bytes(raw_config_data.as_slice())?;
            Ok(wifi_config)
        }
        else {
            Ok(WifiConfig::default())
        }
    }

    fn save<S: Storage>(&self, storage: &mut S) -> Result<(), Error> {
        let buf = postcard::to_stdvec(self)?;
        storage.put_raw(WIFI_CONFIG_KEY, buf)?;
        Ok(())
    }
}

