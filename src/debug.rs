//! Reads data from a TCP socket or serial port and prints the raw sml
//! messages to stdout, without publishing anything to MQTT.

use crate::error::WattwolfError;
use sml_rs::parser::complete::File;
use std::io::Read;

pub fn run(reader: Box<dyn Read>) -> Result<(), WattwolfError> {
    let mut sml_reader = sml_rs::SmlReader::from_reader(reader);

    loop {
        let file = sml_reader
            .next::<File>()
            .ok_or(WattwolfError::UnexpectedEndOfStream)?
            .map_err(WattwolfError::ReadError)?;

        tracing::info!("{file:#?}");
    }
}
