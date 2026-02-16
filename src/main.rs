//! Reads data from a TCP socket or serial port and prints the contained sml messages to stdout

use crate::config::{Config, Connection};
use crate::error::WattwolfError;
use serialport::{Parity, StopBits};
use sml_rs::parser::complete::File;
use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

mod config;
mod error;
mod model;
mod obis;

fn main() -> Result<(), WattwolfError> {
    let config = Config::from_file("config.toml")?;

    println!("Config: {config:#?}");

    // Create reader based on connection type
    let reader: Box<dyn Read> = match &config.connection {
        Connection::Socket(addr) => {
            Box::new(TcpStream::connect(addr).map_err(WattwolfError::ConnectionError)?)
        }
        Connection::Device(device) => {
            let port = serialport::new(device, 9_600)
                .stop_bits(StopBits::One)
                .parity(Parity::None)
                .timeout(Duration::from_millis(5000))
                .open()
                .map_err(WattwolfError::SerialPortError)?;
            Box::new(port)
        }
    };

    let mut sml_reader = sml_rs::SmlReader::from_reader(reader);

    loop {
        let measurement = sml_reader
            .next::<File>()
            .ok_or(WattwolfError::UnexpectedEndOfStream)
            .and_then(|res| -> Result<_, WattwolfError> {
                let file = res.map_err(WattwolfError::ReadError)?;
                Ok(file)
            });

        match measurement {
            Ok(measurement) => println!("Measurement: {measurement:#?}"),
            Err(e) => eprintln!("Error reading measurement: {e}"),
        }
    }
}
