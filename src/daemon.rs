//! Reads data from a TCP socket or serial port, publishes contained sml
//! measurements to MQTT and maintains Home Assistant auto-discovery/availability.

use crate::config::Config;
use crate::error::WattwolfError;
use crate::wire::wire;
use crate::{model, mqtt};
use sml_rs::parser::complete::File;
use std::collections::HashMap;
use std::io::Read;
use std::thread::sleep;
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

pub fn run(config: &Config, reader: Box<dyn Read>) -> Result<(), WattwolfError> {
    let (mqtt_publisher, handler, connection) = wire(config);

    mqtt::eventloop::spawn(connection, handler);

    info!(
        host = %config.mqtt.host,
        port = config.mqtt.port,
        "Connected to MQTT broker"
    );

    let mut sml_reader = sml_rs::SmlReader::from_reader(reader);

    // Track last published values and timestamp
    let mut last_published_values: HashMap<String, u64> = HashMap::new();
    let mut last_publish_time = Instant::now();
    let min_publish_interval = Duration::from_secs(config.mqtt.min_publish_interval);
    let max_publish_interval = Duration::from_secs(config.mqtt.max_publish_interval);

    let mut err_count = 0;

    loop {
        let measurements = sml_reader
            .next::<File>()
            .ok_or(WattwolfError::UnexpectedEndOfStream)
            .and_then(|res| -> Result<_, WattwolfError> {
                let file = res.map_err(WattwolfError::ReadError)?;

                let measurements = model::read_measurement(&file, config.sensors.as_slice())?;
                Ok(measurements)
            });

        match measurements {
            Err(e) if 6 < err_count => {
                error!(error = %e, "Error reading measurement 6 consecutive times, crashing");
                // crash, let systemd restart
                return Err(e);
            }
            Err(e) => {
                error!(error = %e, "Error reading measurement");
                err_count += 1;
                sleep(Duration::from_secs(2 << err_count));
            }
            Ok(measurements) => {
                if 0 < err_count {
                    tracing::info!("Recovered from {err_count} failed measurements");
                    err_count = 0;
                }
                // Check if any value has changed…
                let any_value_changed = measurements.iter().any(|m| {
                    last_published_values
                        .get(&m.sensor.name)
                        .is_none_or(|&last_val| last_val != m.value)
                });

                // …or maximum publish interval has elapsed
                let max_interval_elapsed = max_publish_interval < last_publish_time.elapsed();

                // don't publish more often than min_publish_interval
                let min_interval_elapsed = min_publish_interval <= last_publish_time.elapsed();

                if (any_value_changed && min_interval_elapsed) || max_interval_elapsed {
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
