use crate::model::MeasurementError;
use sml_rs::ReadParsedError;
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

    #[error(transparent)]
    Sml(#[from] MeasurementError),
}
