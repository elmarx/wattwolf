//! Reads data from a serial port and prints the contained sml messages to stdout

use crate::error::WattwolfError;
use crate::model::Measurement;
use serialport::{Parity, StopBits};
use sml_rs::parser::complete::File;
use std::time::Duration;

mod error;
mod model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: read the port from CLI args/env
    let ports = serialport::available_ports()?;
    let port_name = "/dev/ttyUSB0";

    println!("Connecting to port {}", port_name);
    let port = serialport::new(port_name, 9_600)
        .stop_bits(StopBits::One)
        .parity(Parity::None)
        .timeout(Duration::from_millis(5000))
        .open()?;

    let mut reader = sml_rs::SmlReader::from_reader(port);

    loop {
        let measurement = reader
            .next::<File>()
            .ok_or(WattwolfError::UnexpectedEndOfStream)
            .and_then(|res| -> Result<Measurement, WattwolfError> {
                let res = res.map_err(WattwolfError::ReadError)?;
                let measurement = Measurement::try_from(res)?;

                Ok(measurement)
            });

        match measurement {
            Ok(measurement) => println!(
                "Measurement: consumed={} produced={}",
                measurement.consumed, measurement.produced
            ),
            Err(e) => eprintln!("Error reading measurement: {e}"),
        }
    }
}
