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
    pub mqtt: MqttConfig,
    pub sensors: Vec<Sensor>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MqttConfig {
    pub host: String,
    #[serde(default = "default_mqtt_port")]
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default = "default_topic_prefix")]
    pub topic_prefix: String,
    #[serde(default = "default_availability_topic")]
    pub availability_topic: String,
}

impl MqttConfig {
    /// Override username and password from environment variables if set
    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(username) = std::env::var("MQTT_USERNAME") {
            self.username = Some(username);
        }
        if let Ok(password) = std::env::var("MQTT_PASSWORD") {
            self.password = Some(password);
        }

        if let Ok(host) = std::env::var("MQTT_HOST") {
            self.host = host;
        }

        self
    }
}

fn default_mqtt_port() -> u16 {
    1883
}

fn default_topic_prefix() -> String {
    "wattwolf".to_string()
}

fn default_availability_topic() -> String {
    "wattwolf/availability".to_string()
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, strum::AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DeviceClass {
    Power,
    Energy,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, WattwolfError> {
        let contents = fs::read_to_string(path).map_err(WattwolfError::ConfigFileReadError)?;
        let mut config: Config =
            toml::from_str(&contents).map_err(WattwolfError::ConfigParseError)?;

        config.mqtt = config.mqtt.with_env_overrides();

        Ok(config)
    }
}
