//! Reads data from a TCP socket or serial port and prints the contained sml messages to stdout

use crate::config::{Config, Connection};
use crate::error::WattwolfError;
use serialport::{Parity, StopBits};
use sml_rs::parser::complete::File;
use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

mod config;
mod error;
mod model;
mod mqtt;
mod obis;

fn main() -> Result<(), WattwolfError> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_file("config.toml")?;

    info!(?config, "Loaded configuration");

    // Create MQTT publisher
    let mqtt_publisher = mqtt::MqttPublisher::new(&config.mqtt);
    mqtt_publisher.run();
    info!(
        host = %config.mqtt.host,
        port = config.mqtt.port,
        "Connected to MQTT broker"
    );

    // Publish Home Assistant auto-discovery messages
    mqtt_publisher.publish_discovery(&config.sensors)?;
    info!(
        sensor_count = config.sensors.len(),
        "Published auto-discovery messages"
    );

    // Set device status to online
    mqtt_publisher.set_online()?;
    info!("Device status set to online");

    // Create reader based on connection type
    let reader: Box<dyn Read> = match &config.connection {
        Connection::Socket(addr) => {
            info!(address = %addr, "Connecting to TCP socket");
            Box::new(TcpStream::connect(addr).map_err(WattwolfError::ConnectionError)?)
        }
        Connection::Device(device) => {
            info!(device = %device, "Opening serial port");
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

    // Track last published values and timestamp
    let mut last_published_values: HashMap<String, u64> = HashMap::new();
    let mut last_publish_time = Instant::now();
    let min_publish_interval = Duration::from_secs(config.mqtt.min_publish_interval);

    loop {
        let measurements = sml_reader
            .next::<File>()
            .ok_or(WattwolfError::UnexpectedEndOfStream)
            .and_then(|res| -> Result<_, WattwolfError> {
                let file = res.map_err(WattwolfError::ReadError)?;

                if config.debug {
                    debug!(?file, "Raw SML file");
                }

                let measurements = model::read_measurement(&file, config.sensors.as_slice())?;
                Ok(measurements)
            });

        match measurements {
            Err(e) => error!(error = %e, "Error reading measurement"),
            Ok(measurements) => {
                // Check if any value has changed…
                let any_value_changed = measurements.iter().any(|m| {
                    last_published_values
                        .get(&m.sensor.name)
                        .is_none_or(|&last_val| last_val != m.value)
                });

                // …or minimum publish interval has elapsed
                let time_elapsed = min_publish_interval < last_publish_time.elapsed();

                if any_value_changed || time_elapsed {
                    // Publish each measurement to MQTT
                    for measurement in &measurements {
                        if let Err(e) = mqtt_publisher
                            .publish_measurement(&measurement.sensor.name, measurement.value)
                        {
                            error!(
                                error = %e,
                                sensor = %measurement.sensor.name,
                                "Error publishing measurement to MQTT"
                            );
                        } else {
                            debug!(
                                sensor = %measurement.sensor.name,
                                value = measurement.value,
                                "Published measurement"
                            );
                            // Update last published value
                            last_published_values
                                .insert(measurement.sensor.name.clone(), measurement.value);

                            // Update last publish time
                            last_publish_time = Instant::now();
                        }
                    }
                }
            }
        }
    }
}
