use sml_rs::ReadParsedError;
use sml_rs::parser::OctetStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WattwolfError {
    #[error("Could not connect: {0}")]
    ConnectionError(#[source] std::io::Error),

    #[error("Could not read from port: {0}")]
    ReadError(#[source] ReadParsedError<std::io::Error>),

    #[error("Unexpected end of stream")]
    UnexpectedEndOfStream,

    #[error("Serial port error: {0}")]
    SerialPortError(#[source] serialport::Error),

    #[error("Failed to read configuration file: {0}")]
    ConfigFileReadError(#[source] std::io::Error),

    #[error("Failed to parse configuration file: {0}")]
    ConfigParseError(#[source] toml::de::Error),

    #[error("Failed to publish MQTT HomeAssistant sensor autodiscovery message: {0}")]
    MqttHaDiscovery(#[source] rumqttc::ClientError),

    #[error("Failed to publish MQTT sensor measurement: {0}")]
    MqttSensorPublish(#[source] rumqttc::ClientError),

    #[error("Failed to publish MQTT availability message: {0}")]
    MqttOnlinePublish(#[source] rumqttc::ClientError),

    #[error(transparent)]
    Sml(#[from] MeasurementError),

    #[error("Config nof found")]
    ConfigNotFound,
}

#[derive(Error, Debug)]
pub enum MeasurementError {
    #[error("required message `GetListResponse` not found")]
    ListResponseNotFound,
    #[error("got a ListEntry with an unexpected value type for OBIS-type `{0:?}`")]
    UnexpectedValueType(OctetStr<'static>),
    /// for obis sensors there should be a fixed value id. Precision should be set via scaler.
    #[error("got unit id {0:?}, expected {1}")]
    UnexpectedUnitId(Option<u8>, u8),
    #[error("required value for OBIS-type `{0:?}` not found")]
    MissingValue(OctetStr<'static>),
}
