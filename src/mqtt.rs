use crate::config::{MqttConfig, Sensor};
use crate::error::WattwolfError;
use rumqttc::{Client, Connection, LastWill, MqttOptions, QoS};
use serde_json::json;
use std::cell::RefCell;
use std::time::Duration;
use tracing::{error, info};

fn handle_connection(mut connection: Connection, client: &Client, availability_topic: &str) {
    const INITIAL_BACKOFF: Duration = Duration::from_millis(100);
    const MAX_BACKOFF: Duration = Duration::from_mins(1);
    const BACKOFF_MULTIPLIER: u32 = 2;

    let mut current_backoff = INITIAL_BACKOFF;

    for notification in connection.iter() {
        match notification {
            Err(e) => {
                error!(error = %e, backoff_ms = current_backoff.as_millis(), "MQTT connection error, backing off");

                std::thread::sleep(current_backoff);

                // Exponentially increase backoff, capped at MAX_BACKOFF
                current_backoff = std::cmp::min(
                    current_backoff.saturating_mul(BACKOFF_MULTIPLIER),
                    MAX_BACKOFF,
                );
            }
            Ok(rumqttc::Event::Incoming(rumqttc::Incoming::ConnAck(_)))
                if INITIAL_BACKOFF < current_backoff =>
            {
                // Reset backoff on successful notification
                current_backoff = INITIAL_BACKOFF;

                publish_online(client, availability_topic);
            }
            // this should be the initial "connection up"
            Ok(rumqttc::Event::Incoming(rumqttc::Incoming::ConnAck(_))) => {
                publish_online(client, availability_topic);
            }
            Ok(_) => {}
        }
    }
}

/// Publish "online" to the availability topic using a raw client handle.
fn publish_online(client: &Client, availability_topic: &str) {
    if let Err(e) = client
        .publish(availability_topic, QoS::AtLeastOnce, true, "online")
        .map_err(WattwolfError::MqttOnlinePublish)
    {
        // if the connection is dead there is not much we can/have to do, so just log.
        error!(error = %e, "Publishing \"online\" state failed");
    } else {
        info!("Device status set to online");
    }
}

pub struct MqttPublisher {
    client: Client,
    topic_prefix: String,
    // this RefCell construct is required to separate "new" and "run", since rumqttc "new" returns a connection.
    connection: RefCell<Option<Connection>>,
    availability_topic: String,
}

impl MqttPublisher {
    pub fn new(config: &MqttConfig) -> Self {
        let mut mqtt_options = MqttOptions::new("wattwolf", &config.host, config.port);
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            mqtt_options.set_credentials(username, password.0.as_str());
        }

        // Configure Last Will and Testament to set device offline when connection is lost
        let last_will = LastWill::new(
            &config.availability_topic,
            "offline",
            QoS::AtLeastOnce,
            true,
        );
        mqtt_options.set_last_will(last_will);

        let (client, connection) = Client::new(mqtt_options, 10);

        Self {
            client,
            topic_prefix: config.topic_prefix.clone(),
            availability_topic: config.availability_topic.clone(),
            connection: RefCell::new(Some(connection)),
        }
    }

    /// poll the eventloop. Only call this method once
    pub fn run(&self) {
        let connection = self
            .connection
            .borrow_mut()
            .take()
            .expect("expected connection");

        // Clone the cheap, thread-safe client handle and topic instead of
        // sharing `&self` (which holds a `RefCell` and isn't `Sync`).
        let client = self.client.clone();
        let availability_topic = self.availability_topic.clone();

        // Spawn a thread to handle the connection
        std::thread::spawn(move || {
            handle_connection(connection, &client, &availability_topic);
        });
    }

    /// Publish Home Assistant auto-discovery messages for all sensors
    pub fn publish_discovery(&self, sensors: &[Sensor]) -> Result<(), WattwolfError> {
        for sensor in sensors {
            self.publish_sensor_discovery(sensor)?;
        }
        Ok(())
    }

    /// Publish Home Assistant auto-discovery message for a single sensor
    fn publish_sensor_discovery(&self, sensor: &Sensor) -> Result<(), WattwolfError> {
        // device class as configured by the user
        let device_class = sensor
            .device_class
            .as_ref()
            .map(std::convert::AsRef::as_ref);
        // unit is hardcoded given the obis numbers
        let unit = sensor.obis.unit.as_ref();

        // see discovery payload: https://www.home-assistant.io/integrations/mqtt/#single-component-discovery-payload
        let discovery_payload = json!({
            "name": sensor.friendly_name,
            "unique_id": format!("wattwolf_{}", sensor.name),
            "state_topic": format!("{}/{}/state", self.topic_prefix, sensor.name),
            "unit_of_measurement": unit,
            "device_class": device_class,
            "state_class": sensor.state_class.as_deref(),
            "availability_topic": self.availability_topic,
            "payload_available": "online",
            "payload_not_available": "offline",
            "device": {
                "identifiers": ["wattwolf"],
                "name": "Wattwolf Smart Meter",
                "model": "SML Reader",
                "manufacturer": "Wattwolf"
            }
        });

        // see discovery topic documentation here: https://www.home-assistant.io/integrations/mqtt/#discovery-topic
        let discovery_topic = format!("homeassistant/sensor/wattwolf/{}/config", sensor.name);

        self.client
            .publish(
                discovery_topic,
                QoS::AtLeastOnce,
                false,
                discovery_payload.to_string(),
            )
            .map_err(WattwolfError::MqttHaDiscovery)?;

        Ok(())
    }

    /// Publish a measurement value for a sensor
    pub fn publish_measurement(&self, sensor_name: &str, value: u64) -> Result<(), WattwolfError> {
        let state_topic = format!("{}/{}/state", self.topic_prefix, sensor_name);

        self.client
            .publish(state_topic, QoS::AtLeastOnce, true, value.to_string())
            .map_err(WattwolfError::MqttSensorPublish)?;

        Ok(())
    }
}
