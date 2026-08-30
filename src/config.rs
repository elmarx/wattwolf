use crate::error::WattwolfError;
use crate::obis::ObisKeyFigure;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// A wrapper type for sensitive values that hides the content when formatted with Debug
#[derive(Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Secret<T>(pub T);

impl<T> std::fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("***")
    }
}

impl<T> std::ops::Deref for Secret<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
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
    pub password: Option<Secret<String>>,
    #[serde(default = "default_topic_prefix")]
    pub topic_prefix: String,
    #[serde(default = "default_availability_topic")]
    pub availability_topic: String,
    /// Minimum interval in seconds to publish values, i.e.: do not send intervals more often than this value
    #[serde(default = "default_min_publish_interval")]
    pub min_publish_interval: u64,
    /// Maximum interval in seconds to publish values, i.e.: publish values at least every `max_publish_interval` seconds
    #[serde(default = "default_max_publish_interval")]
    pub max_publish_interval: u64,

    /// homeassistant topic, used for the discovery topic prefix, defaults to homeassistant
    #[serde(default = "default_homeassistant_topic")]
    pub homeassistant_topic: String,
}

impl MqttConfig {
    /// Override username and password from environment variables if set
    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(username) = std::env::var("MQTT_USERNAME") {
            self.username = Some(username);
        }
        if let Ok(password) = std::env::var("MQTT_PASSWORD") {
            self.password = Some(Secret(password));
        } else if let Ok(password_file) = std::env::var("MQTT_PASSWORD_FILE") {
            // supports e.g. systemd's LoadCredential=/Environment=..%d/.. mechanism,
            // where the secret is provided as a file instead of directly as an
            // environment variable
            if let Ok(password) = fs::read_to_string(password_file) {
                self.password = Some(Secret(password.trim_end().to_string()));
            }
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

fn default_homeassistant_topic() -> String {
    "homeassistant".to_string()
}

fn default_min_publish_interval() -> u64 {
    10
}

fn default_max_publish_interval() -> u64 {
    300
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

pub fn get_config_path(config_from_cli: Option<PathBuf>) -> Option<PathBuf> {
    config_from_cli
        .or_else(|| std::env::var("WATTWOLF_CONFIG").map(PathBuf::from).ok())
        .or_else(|| {
            let config_in_cwd = PathBuf::from("config.toml");
            config_in_cwd.exists().then_some(config_in_cwd)
        })
        .or_else(|| {
            let config_in_etc = PathBuf::from("/etc/wattwolf.toml");
            config_in_etc.exists().then_some(config_in_etc)
        })
}
