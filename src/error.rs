use crate::model::MeasurementError;
use sml_rs::ReadParsedError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WattwolfError {
    #[error("Could not read from port: {0}")]
    ReadError(#[source] ReadParsedError<std::io::Error>),

    #[error("Unexpected end of stream")]
    UnexpectedEndOfStream,

    #[error(transparent)]
    Sml(#[from] MeasurementError),
}
