use crate::config::{MqttConfig, Sensor};
use crate::error::WattwolfError;
use rumqttc::{Client, Connection, LastWill, MqttOptions, QoS};
use serde_json::json;
use std::cell::RefCell;
use std::time::Duration;

pub struct MqttPublisher {
    client: Client,
    topic_prefix: String,
    connection: RefCell<Option<Connection>>,
    availability_topic: String,
}

impl MqttPublisher {
    pub fn new(config: &MqttConfig) -> Self {
        let mut mqtt_options = MqttOptions::new("wattwolf", &config.host, config.port);
        mqtt_options.set_keep_alive(Duration::from_secs(30));

        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            mqtt_options.set_credentials(username, password);
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
        let mut connection = self
            .connection
            .borrow_mut()
            .take()
            .expect("expected connection");

        // Spawn a thread to handle the connection
        std::thread::spawn(move || {
            for notification in connection.iter() {
                if let Err(e) = notification {
                    eprintln!("MQTT connection error: {e}");
                }
            }
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
                true,
                discovery_payload.to_string(),
            )
            .map_err(WattwolfError::MqttHaDiscovery)?;

        Ok(())
    }

    /// Publish a measurement value for a sensor
    pub fn publish_measurement(&self, sensor_name: &str, value: u64) -> Result<(), WattwolfError> {
        let state_topic = format!("{}/{}/state", self.topic_prefix, sensor_name);

        self.client
            .publish(state_topic, QoS::AtLeastOnce, false, value.to_string())
            .map_err(WattwolfError::MqttSensorPublish)?;

        Ok(())
    }

    /// Publish online status to the availability topic
    pub fn set_online(&self) -> Result<(), WattwolfError> {
        self.client
            .publish(&self.availability_topic, QoS::AtLeastOnce, true, "online")
            .map_err(WattwolfError::MqttOnlinePublish)?;

        Ok(())
    }
}
