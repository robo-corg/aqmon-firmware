use serde::{Deserialize, Serialize};

use crate::config::WifiConfig;

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    SetWifiConfig(WifiConfig)
}

