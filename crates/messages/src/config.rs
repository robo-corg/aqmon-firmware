use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WifiConfig {
    pub ssid: String,
    pub password: String
}
