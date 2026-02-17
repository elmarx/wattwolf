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
mod mqtt;
mod obis;

fn main() -> Result<(), WattwolfError> {
    let config = Config::from_file("config.toml")?;

    println!("Config: {config:#?}");

    // Create MQTT publisher
    let mqtt_publisher = mqtt::MqttPublisher::new(&config.mqtt);
    mqtt_publisher.run();
    println!(
        "Connected to MQTT broker at {}:{}",
        config.mqtt.host, config.mqtt.port
    );
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
        let measurements = sml_reader
            .next::<File>()
            .ok_or(WattwolfError::UnexpectedEndOfStream)
            .and_then(|res| -> Result<_, WattwolfError> {
                let file = res.map_err(WattwolfError::ReadError)?;

                if config.debug {
                    println!("Raw SML file: {file:#?}");
                }

                let measurements = model::read_measurement(&file, config.sensors.as_slice())?;
                Ok(measurements)
            });

        match measurements {
            Ok(measurements) => {
                // Publish each measurement to MQTT
                for measurement in &measurements {
                    if let Err(e) = mqtt_publisher
                        .publish_measurement(&measurement.sensor.name, measurement.value)
                    {
                        eprintln!("Error publishing measurement to MQTT: {e}");
                    }
                }
            }
            Err(e) => eprintln!("Error reading measurement: {e}"),
        }
    }
}
