use crate::error::WattwolfError;
use crate::obis::ObisKeyFigure;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub debug: bool,
    pub connection: Connection,
    pub sensors: Vec<Sensor>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Connection {
    Device(String),
    Socket(String),
}

impl Default for Connection {
    fn default() -> Self {
        Connection::Socket("localhost:2000".to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Sensor {
    pub name: String,
    pub friendly_name: String,
    #[serde(with = "crate::obis")]
    pub obis: &'static ObisKeyFigure,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_class: Option<DeviceClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_class: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceClass {
    Power,
    Energy,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, WattwolfError> {
        let contents = fs::read_to_string(path).map_err(WattwolfError::ConfigFileReadError)?;
        let config: Config = toml::from_str(&contents).map_err(WattwolfError::ConfigParseError)?;
        Ok(config)
    }
}
